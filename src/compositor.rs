//! TODO fill this in

use manager::{InputManager, InputManagerHandler, OutputManager, OutputManagerHandler};
use std::env;
use std::ffi::CStr;
use wayland_sys::server::{WAYLAND_SERVER_HANDLE, wl_display, wl_event_loop};
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_backend, wlr_backend_autocreate, wlr_backend_destroy, wlr_backend_start};

pub struct Compositor {
    input_manager: InputManager,
    output_manager: OutputManager,
    backend: *mut wlr_backend,
    display: *mut wl_display,
    event_loop: *mut wl_event_loop
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
            let display = ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                        wl_display_create,) as
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
            // TODO This is the segfault...
            wl_signal_add(&mut (*backend).events.input_add as *mut _ as _,
                          &mut input_manager.add_listener as *mut _ as _);
            wl_signal_add(&mut (*backend).events.input_remove as *mut _ as _,
                          &mut input_manager.remove_listener as *mut _ as _);
            wl_signal_add(&mut (*backend).events.output_add as *mut _ as _,
                          &mut output_manager.add_listener as *mut _ as _);
            wl_signal_add(&mut (*backend).events.output_remove as *mut _ as _,
                          &mut output_manager.remove_listener as *mut _ as _);

            let socket = ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_display_add_socket_auto, display);
            if socket.is_null() {
                // NOTE Rationale for panicking:
                // * Won't be in C land just yet, so it's safe to panic
                // * Can always be returned in a Result instead, but for now
                //   if you auto create it's assumed you can't recover.
                panic!("Unable to open wayland socket");
            }
            let socket_name = CStr::from_ptr(socket).to_string_lossy().into_owned();
            wlr_log!(L_DEBUG, "Running compositor on wayland display {}",
                     socket_name);
            // TODO Why am I doing this again? It's because of nesting, there's
            // an issue somewhere highlighting why this is the way it is
            env::set_var("_WAYLAND_DISPLAY", socket_name);
            Compositor {
                input_manager,
                output_manager,
                backend,
                display,
                event_loop
            }
        }
    }

    /// Enters the wayland event loop. Won't return until the compositor is
    /// shut off
    // TODO Return ! ?
    pub fn run(&mut self) {
        unsafe {
            wlr_log!(L_INFO, "Starting compositor");
            if !wlr_backend_start(self.backend) {
                wlr_backend_destroy(self.backend);
                // NOTE Rationale for panicking:
                // * Won't be in C land just yet, so it's safe to panic
                // * Can always be returned in a Result instead, but for now
                //   if you auto create it's assumed you can't recover.
                panic!("Failed to start backend");
            }
            ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_display_run, self.display);
        }
        // TODO Clean up
    }
}
