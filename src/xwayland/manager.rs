//! Global manager for the XWayland server.

use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::wlr_xwayland_surface;

use libc;

use types::surface::InternalSurfaceState;
use super::surface::{XWaylandShell, XWaylandSurface, XWaylandSurfaceHandle, XWaylandSurfaceHandler, XWaylandSurfaceState };
use compositor::{compositor_handle, CompositorHandle};
use SurfaceHandler;

pub trait XWaylandManagerHandler {
    /// Callback that's triggered when the XWayland library is ready.
    fn on_ready(&mut self, CompositorHandle) {}

    /// Callback that's triggered when a new surface is presented to the X
    /// server.
    fn new_surface(&mut self,
                   CompositorHandle,
                   XWaylandSurfaceHandle)
                   -> Option<(Box<XWaylandSurfaceHandler>, Box<SurfaceHandler>)> {
        None
    }
}

wayland_listener!(XWaylandManager, Box<XWaylandManagerHandler>, [
    on_ready_listener => on_ready_notify: |this: &mut XWaylandManager, _data: *mut libc::c_void,|
    unsafe {
        let ref mut manager = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_ready(compositor);
    };
    new_surface_listener => new_surface_notify: |this: &mut XWaylandManager,
                                                 data: *mut libc::c_void,|
    unsafe {
        let ref mut manager = this.data;
        let surface_ptr = data as *mut wlr_xwayland_surface;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let shell_surface = XWaylandSurface::new(surface_ptr);
        let res = manager.new_surface(compositor, shell_surface.weak_reference());
        if let Some((xwayland_handler, surface_handler)) = res {
            let mut shell = XWaylandShell::new((shell_surface, xwayland_handler));
            let surface_state = (*(*surface_ptr).surface).data as *mut InternalSurfaceState;
            (*(*surface_state).surface).data().1 = surface_handler;


            wl_signal_add(&mut (*surface_ptr).events.destroy as *mut _ as _,
                          shell.destroy_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.request_configure as *mut _ as _,
                          shell.request_configure_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.request_move as *mut _ as _,
                          shell.request_move_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.request_resize as *mut _ as _,
                          shell.request_resize_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.request_maximize as *mut _ as _,
                          shell.request_maximize_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.request_fullscreen as *mut _ as _,
                          shell.request_fullscreen_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.map as *mut _ as _,
                          shell.map_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.unmap as *mut _ as _,
                          shell.unmap_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.set_title as *mut _ as _,
                          shell.set_title_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.set_class as *mut _ as _,
                          shell.set_class_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.set_parent as *mut _ as _,
                          shell.set_parent_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.set_pid as *mut _ as _,
                          shell.set_pid_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.set_window_type as *mut _ as _,
                          shell.set_window_type_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.ping_timeout as *mut _ as _,
                          shell.ping_timeout_listener() as *mut _ as _);
            let shell_data = (*surface_ptr).data as *mut XWaylandSurfaceState;
            (*shell_data).shell = Box::into_raw(shell);
        }
        // TODO Pass in the new surface from the data
    };
]);
