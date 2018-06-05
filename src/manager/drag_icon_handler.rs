//! Handler for drag icons

use libc;

use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use {SurfaceHandle, CompositorHandle, DragIcon, DragIconHandle};

/// Handles events from the wlr drag icon
pub trait DragIconHandler {
    /// Called when the drag icon is ready to be displayed.
    fn on_map(&mut self, CompositorHandle, SurfaceHandle, DragIconHandle);

    /// Called when the drag icon is about to be destroyed.
    fn destroyed(&mut self, CompositorHandle, DragIconHandle);
}

wayland_listener!(DragIconListener, (DragIcon, Box<DragIconHandler>), [
    destroy_listener => destroy_notify: |this: &mut DragIconListener, data: *mut libc::c_void,| unsafe {
        println!("TODO: call the destroyed handler");
    };
    map_listener => map_notify: |this: &mut DragIconListener, data: *mut libc::c_void,| unsafe {
        println!("TODO: call the map handler");
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
