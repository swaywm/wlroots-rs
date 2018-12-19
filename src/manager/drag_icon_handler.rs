//! Handler for drag icons

use libc;
use wlroots_sys::WAYLAND_SERVER_HANDLE;

use {compositor, seat::drag_icon::{self, DragIcon}};

/// Handles events from the wlr drag icon
#[allow(unused_variables)]
pub trait Handler {
    /// Called when the drag icon is ready to be displayed.
    fn on_map(&mut self,
              compositor_handle: compositor::Handle,
              drag_icon_handle: drag_icon::Handle);

    /// Called when the drag icon should no longer be displayed
    fn on_unmap(&mut self,
                compositor_handle: compositor::Handle,
                drag_icon_handle: drag_icon::Handle);

    /// Called when the drag icon is about to be destroyed.
    fn destroyed(&mut self,
                 compositor_handle: compositor::Handle,
                 drag_icon_handle: drag_icon::Handle);
}

wayland_listener!(pub(crate) Listener, (DragIcon, Box<Handler>), [
    destroy_listener => destroy_notify: |this: &mut Listener, _data: *mut libc::c_void,| unsafe {
        {
            let (ref drag_icon, ref mut handler) = this.data;
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };
            handler.destroyed(compositor, drag_icon.weak_reference());
        }
        Box::from_raw(this);
    };
    map_listener => map_notify: |this: &mut Listener, _data: *mut libc::c_void,| unsafe {
        let (ref drag_icon, ref mut handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        handler.on_map(compositor, drag_icon.weak_reference());
    };
    unmap_listener => unmap_notify: |this: &mut Listener, _data: *mut libc::c_void,| unsafe {
        let (ref drag_icon, ref mut handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        handler.on_unmap(compositor, drag_icon.weak_reference());
    };
]);

impl Drop for Listener {
    fn drop(&mut self) {
        unsafe {
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.destroy_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.map_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.unmap_listener()).link as *mut _ as _);
        }
    }
}
