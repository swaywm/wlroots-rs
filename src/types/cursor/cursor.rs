//! Wrapper for wlr_cursor

use std::{fmt, ptr};

use libc;
use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_cursor, wlr_cursor_absolute_to_layout_coords,
                  wlr_cursor_attach_input_device, wlr_cursor_create, wlr_cursor_destroy,
                  wlr_cursor_detach_input_device, wlr_cursor_map_input_to_output,
                  wlr_cursor_map_input_to_region, wlr_cursor_map_to_output,
                  wlr_cursor_map_to_region, wlr_cursor_move, wlr_cursor_set_image,
                  wlr_cursor_set_surface, wlr_cursor_warp, wlr_cursor_warp_absolute};

use {Area, InputDevice, Output, OutputHandle, OutputLayoutHandle, Surface, XCursorImage};
use compositor::{Compositor, COMPOSITOR_PTR};
use errors::UpgradeHandleErr;
use events::{pointer_events, tablet_tool_events, touch_events};

pub struct CursorBuilder {
    cursor: *mut wlr_cursor,
    cursor_handler: Box<CursorHandler>
}

/// A way to refer to cursors that you want to remove.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct CursorId(*mut wlr_cursor);

impl CursorId {
    pub(crate) fn new(ptr: *mut wlr_cursor) -> Self {
        CursorId(ptr)
    }
}

pub trait CursorHandler {
    /// Callback that is triggered when the cursor moves.
    fn on_pointer_motion(&mut self,
                         &mut Compositor,
                         &mut Cursor,
                         &mut pointer_events::MotionEvent) {
    }

    fn on_pointer_motion_absolute(&mut self,
                                  &mut Compositor,
                                  &mut Cursor,
                                  &mut pointer_events::AbsoluteMotionEvent) {
    }

    /// Callback that is triggered when the buttons on the pointer are pressed.
    fn on_pointer_button(&mut self,
                         &mut Compositor,
                         &mut Cursor,
                         &mut pointer_events::ButtonEvent) {
    }

    fn on_pointer_axis(&mut self, &mut Compositor, &mut Cursor, &mut pointer_events::AxisEvent) {}

    fn on_touch_up(&mut self, &mut Compositor, &mut Cursor, &mut touch_events::UpEvent) {}
    fn on_touch_down(&mut self, &mut Compositor, &mut Cursor, &mut touch_events::DownEvent) {}
    fn on_touch_motion(&mut self, &mut Compositor, &mut Cursor, &mut touch_events::MotionEvent) {}
    fn on_touch_cancel(&mut self, &mut Compositor, &mut Cursor, &mut touch_events::CancelEvent) {}

    fn on_tablet_tool_axis(&mut self,
                           &mut Compositor,
                           &mut Cursor,
                           &mut tablet_tool_events::AxisEvent) {
    }
    fn on_tablet_tool_proximity(&mut self,
                                &mut Compositor,
                                &mut Cursor,
                                &mut tablet_tool_events::ProximityEvent) {
    }
    fn on_tablet_tool_tip(&mut self,
                          &mut Compositor,
                          &mut Cursor,
                          &mut tablet_tool_events::TipEvent) {
    }
    fn on_tablet_tool_button(&mut self,
                             &mut Compositor,
                             &mut Cursor,
                             &mut tablet_tool_events::ButtonEvent) {
    }
}

#[derive(Debug)]
pub struct Cursor {
    cursor: *mut wlr_cursor,
    output_layout: OutputLayoutHandle
}

wayland_listener!(CursorWrapper, (Cursor, Box<CursorHandler>), [
    pointer_motion_listener => pointer_motion_notify:
    |this: &mut CursorWrapper, event: *mut libc::c_void,|
    unsafe {
        let (ref mut cursor, ref mut cursor_handler) = this.data;
        let cursor_id = cursor.id();
        let boxed_cursor = cursor.output_layout.run(|output_layout| {
            output_layout.cursors.borrow_mut().remove(&cursor_id)
                .expect("Cursor already borrowed")})
            .expect("Could not remove cursor from OutputLayout");
        let mut event = pointer_events::MotionEvent::from_ptr(event as _);
        let compositor = &mut *COMPOSITOR_PTR;
        cursor_handler.on_pointer_motion(compositor, cursor, &mut event);
        cursor.output_layout.run(|output_layout|
                                 output_layout.cursors.borrow_mut().insert(cursor_id, boxed_cursor))
            .expect("Could not re-insert cursor to the OutputLayout");
    };
    pointer_motion_absolute_listener => pointer_motion_absolute_notify:
    |this: &mut CursorWrapper, event: *mut libc::c_void,|
    unsafe {
        let (ref mut cursor, ref mut cursor_handler) = this.data;
        let mut event = pointer_events::AbsoluteMotionEvent::from_ptr(event as _);
        let compositor = &mut *COMPOSITOR_PTR;
        cursor_handler.on_pointer_motion_absolute(compositor, cursor, &mut event);
    };
    pointer_button_listener => pointer_button_notify:
    |this: &mut CursorWrapper, event: *mut libc::c_void,|
    unsafe {
        let (ref mut cursor, ref mut cursor_handler) = this.data;
        let mut event = pointer_events::ButtonEvent::from_ptr(event as _);
        let compositor = &mut *COMPOSITOR_PTR;
        cursor_handler.on_pointer_button(compositor, cursor, &mut event);
    };
    pointer_axis_listener => pointer_axis_notify:
    |this: &mut CursorWrapper, event: *mut libc::c_void,|
    unsafe {
        let (ref mut cursor, ref mut cursor_handler) = this.data;
        let mut event = pointer_events::AxisEvent::from_ptr(event as _);
        let compositor = &mut *COMPOSITOR_PTR;
        cursor_handler.on_pointer_axis(compositor, cursor, &mut event);
    };
    touch_up_listener => touch_up_notify: |this: &mut CursorWrapper, event: *mut libc::c_void,|
    unsafe {
        let (ref mut cursor, ref mut cursor_handler) = this.data;
        let mut event = touch_events::UpEvent::from_ptr(event as _);
        let compositor = &mut *COMPOSITOR_PTR;
        cursor_handler.on_touch_up(compositor, cursor, &mut event);
    };
    touch_down_listener => touch_down_notify: |this: &mut CursorWrapper, event: *mut libc::c_void,|
    unsafe {
        let (ref mut cursor, ref mut cursor_handler) = this.data;
        let mut event = touch_events::DownEvent::from_ptr(event as _);
        let compositor = &mut *COMPOSITOR_PTR;
        cursor_handler.on_touch_down(compositor, cursor, &mut event);
    };
    touch_motion_listener => touch_motion_notify:
    |this: &mut CursorWrapper, event: *mut libc::c_void,|
    unsafe {
        let (ref mut cursor, ref mut cursor_handler) = this.data;
        let mut event = touch_events::MotionEvent::from_ptr(event as _);
        let compositor = &mut *COMPOSITOR_PTR;
        cursor_handler.on_touch_motion(compositor, cursor, &mut event);
    };
    touch_cancel_listener => touch_cancel_notify:
    |this: &mut CursorWrapper, event: *mut libc::c_void,|
    unsafe {
        let (ref mut cursor, ref mut cursor_handler) = this.data;
        let mut event = touch_events::CancelEvent::from_ptr(event as _);
        let compositor = &mut *COMPOSITOR_PTR;
        cursor_handler.on_touch_cancel(compositor, cursor, &mut event);
    };
    tablet_tool_axis_listener => tablet_tool_axis_notify:
    |this: &mut CursorWrapper, event: *mut libc::c_void,|
    unsafe {
        let (ref mut cursor, ref mut cursor_handler) = this.data;
        let mut event = tablet_tool_events::AxisEvent::from_ptr(event as _);
        let compositor = &mut *COMPOSITOR_PTR;
        cursor_handler.on_tablet_tool_axis(compositor, cursor, &mut event);
    };
    tablet_tool_proximity_listener => tablet_tool_proximity_notify:
    |this: &mut CursorWrapper, event: *mut libc::c_void,|
    unsafe {
        let (ref mut cursor, ref mut cursor_handler) = this.data;
        let mut event = tablet_tool_events::ProximityEvent::from_ptr(event as _);
        let compositor = &mut *COMPOSITOR_PTR;
        cursor_handler.on_tablet_tool_proximity(compositor, cursor, &mut event);
    };
    tablet_tool_tip_listener => tablet_tool_tip_notify:
    |this: &mut CursorWrapper, event: *mut libc::c_void,|
    unsafe {
        let (ref mut cursor, ref mut cursor_handler) = this.data;
        let mut event = tablet_tool_events::TipEvent::from_ptr(event as _);
        let compositor = &mut *COMPOSITOR_PTR;
        cursor_handler.on_tablet_tool_tip(compositor, cursor, &mut event);
    };
    tablet_tool_button_listener => tablet_tool_button_notify:
    |this: &mut CursorWrapper, event: *mut libc::c_void,|
    unsafe {
        let (ref mut cursor, ref mut cursor_handler) = this.data;
        let mut event = tablet_tool_events::ButtonEvent::from_ptr(event as _);
        let compositor = &mut *COMPOSITOR_PTR;
        cursor_handler.on_tablet_tool_button(compositor, cursor, &mut event);
    };
]);

impl CursorWrapper {
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_cursor {
        self.data.0.as_ptr()
    }

    pub(crate) fn cursor(&mut self) -> &mut Cursor {
        &mut self.data.0
    }
}

impl fmt::Debug for CursorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.data.0)
    }
}

impl CursorBuilder {
    pub fn new(cursor_handler: Box<CursorHandler>) -> Option<Self> {
        unsafe {
            let cursor = wlr_cursor_create();
            if cursor.is_null() {
                None
            } else {
                Some(CursorBuilder { cursor: cursor,
                                     cursor_handler })
            }
        }
    }

    /// Sets the image of the cursor to the image from the XCursor.
    pub fn set_cursor_image(self, image: &XCursorImage) -> Self {
        unsafe {
            let scale = 0.0;
            // NOTE Rationale for why lifetime isn't attached:
            //
            // wlr_cursor_set_image uses gl calls internally, which copies
            // the buffer and so it doesn't matter what happens to the
            // xcursor image after this call.
            wlr_cursor_set_image(self.cursor,
                                 image.buffer.as_ptr(),
                                 image.width as i32,
                                 image.width,
                                 image.height,
                                 image.hotspot_x as i32,
                                 image.hotspot_y as i32,
                                 scale)
        }
        self
    }

    pub(crate) fn build(self, output_layout: OutputLayoutHandle) -> Box<CursorWrapper> {
        let cursor = self.cursor;
        let mut cursor_wrapper = CursorWrapper::new((Cursor { cursor,
                                                              output_layout },
                                                    self.cursor_handler));
        unsafe {
            wl_signal_add(&mut (*cursor).events.motion as *mut _ as _,
                          cursor_wrapper.pointer_motion_listener() as *mut _ as _);
            wl_signal_add(&mut (*cursor).events.motion_absolute as *mut _ as _,
                          cursor_wrapper.pointer_motion_absolute_listener() as *mut _ as _);
            wl_signal_add(&mut (*cursor).events.button as *mut _ as _,
                          cursor_wrapper.pointer_button_listener() as *mut _ as _);
            wl_signal_add(&mut (*cursor).events.axis as *mut _ as _,
                          cursor_wrapper.pointer_axis_listener() as *mut _ as _);
            wl_signal_add(&mut (*cursor).events.touch_up as *mut _ as _,
                          cursor_wrapper.touch_up_listener() as *mut _ as _);
            wl_signal_add(&mut (*cursor).events.touch_down as *mut _ as _,
                          cursor_wrapper.touch_down_listener() as *mut _ as _);
            wl_signal_add(&mut (*cursor).events.touch_motion as *mut _ as _,
                          cursor_wrapper.touch_motion_listener() as *mut _ as _);
            wl_signal_add(&mut (*cursor).events.touch_cancel as *mut _ as _,
                          cursor_wrapper.touch_cancel_listener() as *mut _ as _);
            wl_signal_add(&mut (*cursor).events.tablet_tool_axis as *mut _ as _,
                          cursor_wrapper.tablet_tool_axis_listener() as *mut _ as _);
            wl_signal_add(&mut (*cursor).events.tablet_tool_proximity as *mut _ as _,
                          cursor_wrapper.tablet_tool_proximity_listener() as *mut _ as _);
            wl_signal_add(&mut (*cursor).events.tablet_tool_tip as *mut _ as _,
                          cursor_wrapper.tablet_tool_tip_listener() as *mut _ as _);
            wl_signal_add(&mut (*cursor).events.tablet_tool_button as *mut _ as _,
                          cursor_wrapper.tablet_tool_button_listener() as *mut _ as _);
        }
        cursor_wrapper
    }
}

impl Cursor {
    /// Gets a unique id for this Cursor. This id is used to remove it from the
    /// `OutputLayout`.
    pub fn id(&self) -> CursorId {
        CursorId(unsafe { self.as_ptr() })
    }

    /// Get the coordinates the cursor is located at.
    pub fn coords(&self) -> (f64, f64) {
        unsafe { ((*self.cursor).x, (*self.cursor).y) }
    }

    /// Warp the cursor to the given x and y in layout coordinates. If x and y are
    /// out of the layout boundaries or constraints, no warp will happen.
    ///
    /// `dev` may be passed to respect device mapping constraints. If `dev` is None,
    /// device mapping constraints will be ignored.
    ///
    /// Returns true when the mouse warp was successful.
    pub fn warp<'this, O>(&'this mut self, dev: O, x: f64, y: f64) -> bool
        where O: Into<Option<&'this InputDevice>>
    {
        unsafe {
            let dev_ptr = dev.into().map(|input_device| input_device.as_ptr())
                             .unwrap_or(ptr::null_mut());
            wlr_cursor_warp(self.cursor, dev_ptr, x, y)
        }
    }

    pub fn warp_absolute<'this, O>(&'this mut self, dev: O, x_mm: f64, y_mm: f64)
        where O: Into<Option<&'this InputDevice>>
    {
        unsafe {
            let dev_ptr = dev.into().map(|input_device| input_device.as_ptr())
                             .unwrap_or(ptr::null_mut());
            wlr_cursor_warp_absolute(self.cursor, dev_ptr, x_mm, y_mm)
        }
    }

    /// Move the cursor in the direction of the given x and y coordinates.
    ///
    /// `dev` may be passed to respect device mapping constraints. If `dev` is None,
    /// device mapping constraints will be ignored.
    pub fn move_to<'this, O>(&'this mut self, dev: O, delta_x: f64, delta_y: f64)
        where O: Into<Option<&'this InputDevice>>
    {
        unsafe {
            let dev_ptr = dev.into().map(|dev| dev.as_ptr())
                             .unwrap_or(ptr::null_mut());
            wlr_cursor_move(self.cursor, dev_ptr, delta_x, delta_y)
        }
    }

    // TODO Allow setting cursor images to arbitrary bytes,
    // just like in wlroots. Want to make sure that's safe though

    /// Sets the image of the cursor to the image from the XCursor.
    pub fn set_cursor_image(&mut self, image: &XCursorImage) {
        unsafe {
            let scale = 0.0;
            // NOTE Rationale for why lifetime isn't attached:
            //
            // wlr_cursor_set_image uses gl calls internally, which copies
            // the buffer and so it doesn't matter what happens to the
            // xcursor image after this call.
            wlr_cursor_set_image(self.cursor,
                                 image.buffer.as_ptr(),
                                 image.width as i32,
                                 image.width,
                                 image.height,
                                 image.hotspot_x as i32,
                                 image.hotspot_y as i32,
                                 scale)
        }
    }

    /// Set the cursor surface. The surface can be committed to update the cursor
    /// image. The surface position is substracted from the hotspot.
    ///
    /// A `None` surface commit hides the cursor.
    pub fn set_surface<'this, O>(&'this mut self, surface: O, hotspot_x: i32, hotspot_y: i32)
        where O: Into<Option<&'this Surface>>
    {
        unsafe {
            let surface_ptr = surface.into()
                                     .map(|surface| surface.as_ptr())
                                     .unwrap_or(ptr::null_mut());
            wlr_cursor_set_surface(self.cursor, surface_ptr, hotspot_x, hotspot_y)
        }
    }

    /// Attaches this input device to this cursor. The input device must be one of:
    ///
    /// - WLR_INPUT_DEVICE_POINTER
    /// - WLR_INPUT_DEVICE_TOUCH
    /// - WLR_INPUT_DEVICE_TABLET_TOOL
    ///
    /// TODO Make this impossible to mess up with using an enum
    /// Note that it's safe to use the wrong type.
    pub fn attach_input_device(&mut self, dev: &InputDevice) {
        // NOTE Rationale for not storing handle:
        //
        // Internally, on the destroy event this will automatically
        // destroy the internal wlr_cursor_device used to refer to
        // this InputDevice.
        unsafe { wlr_cursor_attach_input_device(self.cursor, dev.as_ptr()) }
    }

    /// Deattaches the input device from this cursor.
    pub fn deattach_input_device(&mut self, dev: &InputDevice) {
        unsafe { wlr_cursor_detach_input_device(self.cursor, dev.as_ptr()) }
    }

    /// Attaches this cursor to the given output, which must be among the outputs in
    /// the current output_layout for this cursor.
    pub fn map_to_output(&mut self, output: Option<&Output>) {
        match output {
            None => unsafe { wlr_cursor_map_to_output(self.cursor, ptr::null_mut()) },
            Some(output) => {
                if !self.output_in_output_layout(output.weak_reference()) {
                    wlr_log!(L_ERROR, "Tried to map output not in the OutputLayout");
                    return
                }
                unsafe { wlr_cursor_map_to_output(self.cursor, output.as_ptr()) }
            }
        }
    }

    /// Maps all input from a specific input device to a given output.
    ///
    /// The input device must be attached to this cursor
    /// and the output must be among the outputs in the attached output layout.
    pub fn map_input_to_output(&mut self, dev: &InputDevice, output: &Output) {
        // NOTE Rationale for why we don't check input:
        //
        // If the input isn't found, then wlroots prints a diagnostic and
        // returns early (and thus does nothing unsafe).

        if !self.output_in_output_layout(output.weak_reference()) {
            wlr_log!(L_ERROR,
                     "Tried to map input to an output not in the OutputLayout");
            return
        }
        unsafe { wlr_cursor_map_input_to_output(self.cursor, dev.as_ptr(), output.as_ptr()) }
    }

    /// Maps this cursor to an arbitrary region on the associated
    /// wlr_output_layout.
    pub fn map_to_region(&mut self, mut area: Area) {
        unsafe { wlr_cursor_map_to_region(self.cursor, &mut area.0) }
    }

    /// Maps inputs from this input device to an arbitrary region on the associated
    /// wlr_output_layout.
    ///
    /// The input device must be attached to this cursor.
    pub fn map_input_to_region(&mut self, dev: &InputDevice, mut area: Area) {
        // NOTE Rationale for why we don't check input:
        //
        // If the input isn't found, then wlroots prints a diagnostic and
        // returns early (and thus does nothing unsafe).
        unsafe { wlr_cursor_map_input_to_region(self.cursor, dev.as_ptr(), &mut area.0) }
    }

    /// Convert absolute coordinates to layout coordinates for the device.
    ///
    /// Coordinates are in (x, y).
    pub fn absolute_to_layout_coords(&mut self,
                                     dev: &InputDevice,
                                     x_mm: f64,
                                     y_mm: f64,
                                     width_mm: f64,
                                     height_mm: f64)
                                     -> (f64, f64) {
        unsafe {
            let (mut lx, mut ly) = (0.0, 0.0);
            wlr_cursor_absolute_to_layout_coords(self.cursor,
                                                 dev.as_ptr(),
                                                 x_mm,
                                                 y_mm,
                                                 width_mm,
                                                 height_mm,
                                                 &mut lx,
                                                 &mut ly);
            (lx, ly)
        }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_cursor {
        self.cursor
    }

    /// Checks if the output is in the OutputLayout associated with this
    /// cursor.
    ///
    /// If it isn't, or the OutputLayout has been dropped, this returns `false`.
    /// Otherwise it returns `true`.
    fn output_in_output_layout(&mut self, output: OutputHandle) -> bool {
        match self.output_layout.run(|output_layout| {
                                         for (cur_output, _) in output_layout.outputs() {
                                             if cur_output == output {
                                                 return true
                                             }
                                         }
                                         false
                                     }) {
            Ok(res) => res,
            Err(UpgradeHandleErr::AlreadyDropped) => false,
            Err(err) => panic!(err)
        }
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        unsafe { wlr_cursor_destroy(self.cursor) }
    }
}

impl Drop for CursorWrapper {
    fn drop(&mut self) {
        wlr_log!(L_DEBUG, "Dropped {:?}", self);
        unsafe {
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.pointer_motion_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.pointer_motion_absolute_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.pointer_button_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.pointer_axis_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.touch_up_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.touch_down_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.touch_motion_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.touch_cancel_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.tablet_tool_axis_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.tablet_tool_proximity_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.tablet_tool_tip_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.tablet_tool_button_listener()).link as *mut _ as _);
        }
    }
}
