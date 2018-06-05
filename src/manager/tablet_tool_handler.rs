//! Handler for tablet tools

use libc;
use wlroots_sys::wlr_input_device;
use wayland_sys::server::WAYLAND_SERVER_HANDLE;

use {TabletTool, TabletToolHandle};
use compositor::{compositor_handle, CompositorHandle};
use events::tablet_tool_events::{AxisEvent, ButtonEvent, ProximityEvent, TipEvent};

pub trait TabletToolHandler {
    /// Callback that is triggered when an axis event fires
    fn on_axis(&mut self, CompositorHandle, TabletToolHandle, &AxisEvent) {}

    /// Callback that is triggered when a table tool is brought close to the
    /// input source.
    fn on_proximity(&mut self, CompositorHandle, TabletToolHandle, &ProximityEvent) {}

    /// Callback that is triggered when a table tool's tip touches the input
    /// source.
    fn on_tip(&mut self, CompositorHandle, TabletToolHandle, &TipEvent) {}

    /// Callback that is triggered when a button is pressed on the tablet tool.
    fn on_button(&mut self, CompositorHandle, TabletToolHandle, &ButtonEvent) {}

    /// Callback that is triggered when a tablet tool is destroyed.
    fn destroyed(&mut self, CompositorHandle, TabletToolHandle) {}
}

wayland_listener!(TabletToolWrapper, (TabletTool, Box<TabletToolHandler>), [
    on_destroy_listener => on_destroy_notify: |this: &mut TabletToolWrapper, data: *mut libc::c_void,|
    unsafe {
        let input_device_ptr = data as *mut wlr_input_device;
        {
            let (ref mut tool, ref mut tablet_tool_handler) = this.data;
            let compositor = match compositor_handle() {
                Some(handle) => handle,
                None => return
            };
            tablet_tool_handler.destroyed(compositor, tool.weak_reference());
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.on_destroy_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.axis_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.proximity_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.tip_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.button_listener()).link as *mut _ as _);
        Box::from_raw((*input_device_ptr).data as *mut TabletToolWrapper);
    };
    axis_listener => axis_notify: |this: &mut TabletToolWrapper, data: *mut libc::c_void,| unsafe {
        let (ref tool, ref mut handler) = this.data;
        let event = AxisEvent::from_ptr(data as *mut _);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_axis(compositor,
                        tool.weak_reference(),
                        &event);
    };
    proximity_listener => proximity_notify: |this: &mut TabletToolWrapper,
    data: *mut libc::c_void,|
    unsafe {
        let (ref tool, ref mut handler) = this.data;
        let event = ProximityEvent::from_ptr(data as *mut _);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_proximity(compositor,
                             tool.weak_reference(),
                             &event);
    };
    tip_listener => tip_notify: |this: &mut TabletToolWrapper, data: *mut libc::c_void,| unsafe {
        let (ref tool, ref mut handler) = this.data;
        let event = TipEvent::from_ptr(data as *mut _);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_tip(compositor,
                       tool.weak_reference(),
                       &event);
    };
    button_listener => button_notify: |this: &mut TabletToolWrapper, data: *mut libc::c_void,|
    unsafe {
        let (ref tool, ref mut handler) = this.data;
        let event = ButtonEvent::from_ptr(data as *mut _);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_button(compositor,
                          tool.weak_reference(),
                          &event);
    };
]);
