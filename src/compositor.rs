//! Main entry point to the library.
//! See examples for documentation on how to use this struct.

use std::{env, ptr};
use std::any::Any;
use std::cell::UnsafeCell;
use std::ffi::CStr;
use std::collections::HashMap;

use extensions::server_decoration::ServerDecorationManager;
use manager::{InputManager, InputManagerHandler, OutputManager, OutputManagerHandler,
              WlShellManager, WlShellManagerHandler};
use render::GenericRenderer;
use types::seat::Seat;

use wayland_sys::server::{wl_display, wl_event_loop, WAYLAND_SERVER_HANDLE};
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_backend, wlr_backend_autocreate, wlr_backend_destroy, wlr_backend_start,
                  wlr_compositor, wlr_compositor_create, wlr_wl_shell, wlr_wl_shell_create};
use wlroots_sys::wayland_server::sys::wl_display_init_shm;

/// Global compositor pointer, used to refer to the compositor state unsafely.
pub static mut COMPOSITOR_PTR: *mut Compositor = 0 as *mut _;

#[allow(dead_code)]
pub struct Compositor {
    /// User data.
    pub data: Box<Any>,
    /// The list of seats.
    ///
    /// This is stored here due to their complicated memory model.
    ///
    /// Please refer to the `Seat` documentation to learn how to use this.
    seats: HashMap<String, Box<Seat>>,
    /// Manager for the inputs.
    input_manager: Option<Box<InputManager>>,
    /// Manager for the outputs.
    output_manager: Option<Box<OutputManager>>,
    /// Manager for Wayland shells.
    wl_shell_manager: Option<Box<WlShellManager>>,
    /// Pointer to wl_shell global.
    /// If wl_shell_manager is `None`, this value will be `NULL`
    wl_shell_global: *mut wlr_wl_shell,
    /// Pointer to the wlr_compositor
    compositor: *mut wlr_compositor,
    /// Pointer to the wlroots backend in use.
    backend: *mut wlr_backend,
    /// Pointer to the wayland display.
    display: *mut wl_display,
    /// Pointer to the event loop.
    event_loop: *mut wl_event_loop,
    /// Shared memory buffer file descriptor.
    shm_fd: i32,
    /// Name of the Wayland socket that we are binding to.
    socket_name: String,
    /// Optional decoration manager extension.
    pub server_decoration_manager: Option<ServerDecorationManager>,
    /// The renderer used to draw things to the screen.
    pub renderer: Option<GenericRenderer>,
    /// The error from the panic, if there was one.
    panic_error: Option<Box<Any + Send>>
}

pub struct CompositorBuilder {
    gles2: bool,
    server_decoration_manager: bool
}

impl CompositorBuilder {
    pub fn new() -> Self {
        CompositorBuilder { gles2: false,
                            server_decoration_manager: false }
    }

    pub fn gles2(mut self, gles2_renderer: bool) -> Self {
        self.gles2 = gles2_renderer;
        self
    }

    pub fn server_decoration_manager(mut self, server_decoration_manager: bool) -> Self {
        self.server_decoration_manager = server_decoration_manager;
        self
    }

    /// Makes a new compositor that handles the setup of the graphical backend
    /// (e.g, Wayland, X11, or DRM).
    ///
    /// Also automatically opens the socket for clients to communicate to the
    /// compositor with.
    pub fn build_auto<D>(self,
                         data: D,
                         input_manager_handler: Option<Box<InputManagerHandler>>,
                         output_manager_handler: Option<Box<OutputManagerHandler>>,
                         wl_shell_manager_handler: Option<Box<WlShellManagerHandler>>)
                         -> Compositor
        where D: Any + 'static
    {
        unsafe {
            let display =
                ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_display_create,) as *mut wl_display;
            let event_loop =
                ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_display_get_event_loop, display);
            let backend = wlr_backend_autocreate(display as *mut _);
            if backend.is_null() {
                // NOTE Rationale for panicking:
                // * Won't be in C land just yet, so it's safe to panic
                // * Can always be returned in a Result instead, but for now
                //   if you auto create it's assumed you can't recover.
                panic!("Could not auto-create backend");
            }
            // Set up shared memory buffer for Wayland clients.
            let shm_fd = wl_display_init_shm(display as *mut _);
            // Create optional extensions.
            let server_decoration_manager = if self.server_decoration_manager {
                ServerDecorationManager::new(display)
            } else {
                None
            };
            let compositor;
            let renderer = if self.gles2 {
                let gles2 = GenericRenderer::gles2_renderer(backend);
                // Set up wlr_compositor
                let gles2_ptr = gles2.as_ptr();
                compositor = wlr_compositor_create(display as *mut _, gles2_ptr);
                Some(gles2)
            } else {
                compositor = wlr_compositor_create(display as *mut _, ptr::null_mut());
                None
            };

            // Set up input manager, if the user provided it.
            let input_manager = input_manager_handler.map(|handler| {
                let mut input_manager = InputManager::new((vec![], handler));
                wl_signal_add(&mut (*backend).events.new_input as *mut _ as _,
                              input_manager.add_listener() as *mut _ as _);
                input_manager
            });

            // Set up output manager, if the user provided it.
            let output_manager = output_manager_handler.map(|handler| {
                let mut output_manager = OutputManager::new((vec![], handler));
                wl_signal_add(&mut (*backend).events.new_output as *mut _ as _,
                              output_manager.add_listener() as *mut _ as _);
                output_manager
            });

            // Set up wl_shell handler and associated Wayland global,
            // if user provided a manager for it.
            let mut wl_shell_global = ptr::null_mut();
            let wl_shell_manager = wl_shell_manager_handler.map(|handler| {
                wl_shell_global = wlr_wl_shell_create(display as *mut _);
                let mut wl_shell_manager = WlShellManager::new(handler);
                wl_signal_add(&mut (*wl_shell_global).events.new_surface as *mut _ as _,
                              wl_shell_manager.add_listener() as *mut _ as _);
                wl_shell_manager
            });

            // Open the socket to the Wayland server.
            let socket = ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_display_add_socket_auto, display);
            if socket.is_null() {
                // NOTE Rationale for panicking:
                // * Won't be in C land just yet, so it's safe to panic
                // * Can always be returned in a Result instead, but for now
                //   if you auto create it's assumed you can't recover.
                panic!("Unable to open wayland socket");
            }
            let socket_name = CStr::from_ptr(socket).to_string_lossy().into_owned();
            wlr_log!(L_DEBUG,
                     "Running compositor on wayland display {}",
                     socket_name);
            env::set_var("_WAYLAND_DISPLAY", socket_name.clone());
            Compositor { data: Box::new(data),
                         seats: HashMap::new(),
                         socket_name,
                         input_manager,
                         output_manager,
                         wl_shell_manager,
                         wl_shell_global,
                         compositor,
                         backend,
                         display,
                         event_loop,
                         shm_fd,
                         server_decoration_manager,
                         renderer,
                         panic_error: None }
        }
    }
}

impl Compositor {
    /// Enters the wayland event loop. Won't return until the compositor is
    /// shut off
    pub fn run(self) {
        unsafe {
            let compositor = UnsafeCell::new(self);
            if COMPOSITOR_PTR != 0 as _ {
                // NOTE Rationale for panicking:
                // * Nicer than an abort
                // * Not yet in C land
                panic!("A compositor is already running!")
            }
            COMPOSITOR_PTR = compositor.get();
            wlr_log!(L_INFO, "Starting compositor");
            if !wlr_backend_start((*compositor.get()).backend) {
                wlr_backend_destroy((*compositor.get()).backend);
                // NOTE Rationale for panicking:
                // * Won't be in C land just yet, so it's safe to panic
                // * Can always be returned in a Result instead, but for now
                //   if you auto create it's assumed you can't recover.
                panic!("Failed to start backend");
            }
            env::set_var("WAYLAND_DISPLAY", (*COMPOSITOR_PTR).socket_name.clone());
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_display_run,
                          (*compositor.get()).display);
            match (*compositor.get()).panic_error.take() {
                None => {}
                Some(err) => {
                    // A panic occured, now we can re-throw it safely.
                    ::std::panic::resume_unwind(err)
                }
            }
        }
    }

    /// Returns a list of the seats.
    pub fn seat(&mut self, name: &str) -> Option<&mut Box<Seat>> {
        self.seats.get_mut(name)
    }

    /// Adds a seat to the list and then returns a reference to it.
    pub(crate) fn add_seat(&mut self, seat: Box<Seat>) -> &mut Box<Seat> {
        let name = seat.name().expect("Could not get seat name");
        self.seats.insert(name.clone(), seat);
        self.seats.get_mut(name.as_str()).unwrap()
    }

    /// Drops the seat associated with the provided name.
    // TODO FIXME Better result types
    pub fn drop_seat(&mut self, name: &str) -> Result<(), ()> {
        match self.seats.remove(name) {
            None => return Err(()), // TODO Better error here
            Some(_seat) => Ok(())
        }
    }

    /// Takes the seat from the list and returns it.
    ///
    /// In its place it places a borrow, so that we can return it afterwards
    /// using `replace_seat`.
    ///
    /// # Panics
    /// Panics if the `SeatId` is invalid or already borrowed.
    // TODO Better errors, don't panic on these things
    pub(crate) fn take_seat(&mut self, name: &str) -> Box<Seat> {
        self.seats.remove(name).expect("Seat did not exist, or was already borrowed")
    }

    /// Replaces the Borrowed in the list with the seat.
    pub(crate) fn replace_seat(&mut self, seat: Box<Seat>) {
        let name = seat.name().expect("Seat had no name");
        self.seats.insert(name, seat);
    }

    pub fn terminate(&mut self) {
        unsafe {
            COMPOSITOR_PTR = 0 as _;
            ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_display_terminate, self.display);
        }
    }

    pub unsafe fn display(&self) -> *mut wl_display {
        self.display
    }

    pub unsafe fn event_loop(&self) -> *mut wl_event_loop {
        self.event_loop
    }

    /// Saves the panic error information in the compositor, to be re-thrown
    /// later when we are out of the C callback stack.
    pub(crate) fn save_panic_error(&mut self, error: Box<Any + Send>) {
        self.panic_error = Some(error);
    }
}

/// Terminates the compositor.
/// If one is not running, does nothing
pub fn terminate() {
    unsafe {
        if COMPOSITOR_PTR != 0 as _ {
            (*COMPOSITOR_PTR).terminate();
            COMPOSITOR_PTR = 0 as _
        }
    }
}
