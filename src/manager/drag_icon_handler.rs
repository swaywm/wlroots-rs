//! Handler for drag icons

use libc;

use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use compositor::{compositor_handle};
use {SurfaceHandle, CompositorHandle, DragIcon, DragIconHandle};

/// Handles events from the wlr drag icon
pub trait DragIconHandler {
    /// Called when the drag icon is ready to be displayed.
    fn on_map(&mut self, CompositorHandle, DragIconHandle);

    /// Called when the drag icon is about to be destroyed.
    fn destroyed(&mut self, CompositorHandle, DragIconHandle);
}

wayland_listener!(DragIconListener, (DragIcon, Box<DragIconHandler>), [
    destroy_listener => destroy_notify: |this: &mut DragIconListener, data: *mut libc::c_void,| unsafe {
        {
            let (ref drag_icon, ref mut handler) = this.data;
            let compositor = match compositor_handle() {
                Some(handle) => handle,
                None => return
            };
            handler.destroyed(compositor, drag_icon.weak_reference());
        }
        Box::from_raw(this);
    };
    map_listener => map_notify: |this: &mut DragIconListener, data: *mut libc::c_void,| unsafe {
        let (ref drag_icon, ref mut handler) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        handler.on_map(compositor, drag_icon.weak_reference());
    };
]);

impl Drop for DragIconListener {
    fn drop(&mut self) {
        unsafe {
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.destroy_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.map_listener()).link as *mut _ as _);
        }
    }
}
