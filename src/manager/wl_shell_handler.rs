//! Handler for Wayland shell clients.

use libc;
use wayland_sys::server::WAYLAND_SERVER_HANDLE;

use WlShellSurface;
use compositor::{Compositor, COMPOSITOR_PTR};

/// Handles events from client Wayland shells.
pub trait WlShellHandler {
    /// Called when the Wayland shell is destroyed (e.g by the user)
    fn destroy(&mut self, &mut Compositor, &mut WlShellSurface) {}

    /// Called when the ping request timed out. This usually indicates something
    /// is wrong with the client
    fn ping_timeout(&mut self, &mut Compositor, &mut WlShellSurface) {}

    /// Called when there is a request to move the shell surface somewhere else.
    fn move_request(&mut self, &mut Compositor, &mut WlShellSurface /* TODO Move event */) {}

    /// Called when there is a request to resize the shell surface.
    fn resize_request(&mut self,
                      &mut Compositor,
                      &mut WlShellSurface /* TODO resize event */) {
    }

    /// Called when there is a request to make the shell surface fullscreen.
    fn fullscreen_request(&mut self,
                          &mut Compositor,
                          &mut WlShellSurface /* TODO Fullscreen event */) {
    }

    /// Called when there is a request to make the shell surface maximized.
    fn maximize_request(&mut self,
                        &mut Compositor,
                        &mut WlShellSurface /* TODO Maximize request */) {
    }

    /// Called when there is a request to change the state of the Wayland shell.
    fn state_change(&mut self, &mut Compositor, &mut WlShellSurface) {}

    /// Called when there is a request to change the title of the Wayland shell.
    fn title_change(&mut self, &mut Compositor, &mut WlShellSurface) {}

    /// Called when there is a request to change the class of the Wayland shell.
    fn class_change(&mut self, &mut Compositor, &mut WlShellSurface) {}
}

wayland_listener!(WlShell, (WlShellSurface, Box<WlShellHandler>), [
    destroy_listener => destroy_notify: |this: &mut WlShell, _data: *mut libc::c_void,|
    unsafe {
        // TODO NLL
        {
            let (ref mut shell_surface, ref mut manager) = this.data;
            let compositor = &mut *COMPOSITOR_PTR;
            shell_surface.set_lock(true);
            manager.destroy(compositor, shell_surface);
            shell_surface.set_lock(false);
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.destroy_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.ping_timeout_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.request_move_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.request_resize_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.request_fullscreen_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.request_maximize_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.set_state_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.set_title_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.set_class_listener()).link as *mut _ as _);
    };
    ping_timeout_listener => ping_timeout_notify: |this: &mut WlShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        shell_surface.set_lock(true);
        manager.ping_timeout(compositor, shell_surface);
        shell_surface.set_lock(false);
    };
    request_move_listener => request_move_notify: |this: &mut WlShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        shell_surface.set_lock(true);
        manager.move_request(compositor, shell_surface);
        shell_surface.set_lock(false);
    };
    request_resize_listener => request_resize_notify: |this: &mut WlShell,
                                                       _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        shell_surface.set_lock(true);
        manager.resize_request(compositor, shell_surface);
        shell_surface.set_lock(false);
    };
    request_fullscreen_listener => request_fullscreen_notify: |this: &mut WlShell,
                                                               _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        shell_surface.set_lock(true);
        manager.fullscreen_request(compositor, shell_surface);
        shell_surface.set_lock(false);
    };
    request_maximize_listener => request_maximize_notify: |this: &mut WlShell,
                                                           _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        shell_surface.set_lock(true);
        manager.maximize_request(compositor, shell_surface);
        shell_surface.set_lock(false);
    };
    set_state_listener => set_state_notify: |this: &mut WlShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        shell_surface.set_lock(true);
        manager.state_change(compositor, shell_surface);
        shell_surface.set_lock(false);
    };
    set_title_listener => set_title_notify: |this: &mut WlShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        shell_surface.set_lock(true);
        manager.title_change(compositor, shell_surface);
        shell_surface.set_lock(false);
    };
    set_class_listener => set_class_notify: |this: &mut WlShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        shell_surface.set_lock(true);
        manager.class_change(compositor, shell_surface);
        shell_surface.set_lock(false);
    };
]);
