//! Main entry point to the library.
//! See examples for documentation on how to use this struct.

use manager::{InputManager, InputManagerHandler, OutputManager, OutputManagerHandler};
use std::cell::UnsafeCell;
use std::env;
use std::ffi::CStr;
use types::server_decoration::ServerDecorationManager;
use wayland_sys::server::{WAYLAND_SERVER_HANDLE, wl_display, wl_event_loop};
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_backend, wlr_backend_autocreate, wlr_backend_destroy, wlr_backend_start};

/// Global compositor pointer, used to refer to the compositor state unsafely.
static mut COMPOSITOR_PTR: *mut Compositor = 0 as *mut _;

#[allow(dead_code)]
pub struct Compositor {
    input_manager: Box<InputManager>,
    output_manager: Box<OutputManager>,
    backend: *mut wlr_backend,
    display: *mut wl_display,
    event_loop: *mut wl_event_loop,
    server_decoration_manager: Option<ServerDecorationManager>
}

impl Compositor {
    /// Makes a new compositor that handles the setup of the graphical backend
    /// (e.g, Wayland, X11, or DRM).
    ///
    /// Also automatically opens the socket for clients to communicate to the
    /// compositor with.
    pub fn new(input_manager_handler: Box<InputManagerHandler>,
               output_manager_handler: Box<OutputManagerHandler>)
               -> Self {
        unsafe {
            let display = ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_display_create,) as
                *mut wl_display;
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
            let mut input_manager = InputManager::new(input_manager_handler);
            let mut output_manager = OutputManager::new(output_manager_handler);
            wl_signal_add(&mut (*backend).events.input_add as *mut _ as _,
                          input_manager.add_listener() as *mut _ as _);
            wl_signal_add(&mut (*backend).events.input_remove as *mut _ as _,
                          input_manager.remove_listener() as *mut _ as _);
            wl_signal_add(&mut (*backend).events.output_add as *mut _ as _,
                          output_manager.add_listener() as *mut _ as _);
            wl_signal_add(&mut (*backend).events.output_remove as *mut _ as _,
                          output_manager.remove_listener() as *mut _ as _);

            let server_decoration_manager = ServerDecorationManager::new(display);

            if server_decoration_manager.is_none() {
                wlr_log!(L_ERROR, "Failed to create server decoration manager");
            }

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
            env::set_var("_WAYLAND_DISPLAY", socket_name);
            Compositor {
                input_manager,
                output_manager,
                backend,
                display,
                event_loop,
                server_decoration_manager
            }
        }
    }

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
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_display_run,
                          (*compositor.get()).display);
        }
        // TODO Clean up
    }

    pub fn server_decoration_manager(&mut self) -> Option<&mut ServerDecorationManager> {
        self.server_decoration_manager.as_mut()
    }

    pub fn terminate(&mut self) {
        unsafe {
            ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_display_terminate, self.display);
        }
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
