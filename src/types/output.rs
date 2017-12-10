use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::{wl_list, wl_output_transform, wlr_output, wlr_output_effective_resolution,
                  wlr_output_events, wlr_output_layout, wlr_output_layout_add_auto,
                  wlr_output_layout_create, wlr_output_layout_destroy, wlr_output_layout_remove,
                  wlr_output_make_current, wlr_output_mode, wlr_output_set_mode,
                  wlr_output_swap_buffers, wlr_output_transform};

use std::cell::RefCell;
use std::ffi::CStr;
use std::rc::Rc;

pub struct OutputState {
    pub layout: Option<Rc<RefCell<OutputLayout>>>
}

/// A wrapper around a wlr_output.
#[derive(Debug)]
pub struct OutputHandle {
    output: *mut wlr_output
}

#[derive(Debug)]
pub struct OutputLayout {
    layout: *mut wlr_output_layout
}

impl OutputHandle {
    pub unsafe fn set_user_data(&mut self, data: Rc<OutputState>) {
        (*self.output).data = Rc::into_raw(data) as *mut _
    }

    pub unsafe fn user_data(&mut self) -> *mut OutputState {
        (*self.output).data as *mut _
    }

    pub fn layout(&mut self) -> Option<Rc<RefCell<OutputLayout>>> {
        unsafe {
            let data = self.user_data();
            if data.is_null() {
                None
            } else {
                (*data).layout.clone()
            }
        }
    }

    pub fn add_layout_auto(&mut self, layout: Rc<RefCell<OutputLayout>>) {
        unsafe {
            wlr_output_layout_add_auto(layout.borrow_mut().to_ptr(), self.output);
            let user_data = self.user_data();
            if user_data.is_null() {
                self.set_user_data(Rc::new(OutputState { layout: Some(layout) }));
            } else {
                (*user_data).layout = Some(layout);
            }
        }
    }

    /// Sets the best modesetting for an output.
    ///
    /// NOTE You _cannot_ call this when the output will be removed.
    /// It must only be called at startup.
    ///
    /// I'm still marking it as safe though because we protect against that
    /// action
    /// in the output destruction callback.
    pub fn choose_best_mode(&mut self) {
        unsafe {
            let length = ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_list_length, self.modes() as _);
            if length > 0 {
                // TODO Better logging
                wlr_log!(L_DEBUG, "output added {:?}", self);
                let first_mode_ptr: *mut wlr_output_mode;
                first_mode_ptr = container_of!(&mut (*(*self.modes()).prev) as *mut _,
                                               wlr_output_mode,
                                               link);
                wlr_output_set_mode(self.to_ptr(), first_mode_ptr);
            }
        }
    }

    /// Gets the name of the output in UTF-8.
    pub fn name(&self) -> String {
        unsafe {
            CStr::from_ptr((*self.output).name.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Gets the make of the output in UTF-8.
    pub fn make(&self) -> String {
        unsafe {
            CStr::from_ptr((*self.output).make.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Gets the model of the output in UTF-8.
    pub fn model(&self) -> String {
        unsafe {
            CStr::from_ptr((*self.output).model.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn make_current(&mut self) {
        unsafe { wlr_output_make_current(self.output) }
    }

    pub fn swap_buffers(&mut self) {
        unsafe { wlr_output_swap_buffers(self.output) }
    }

    /// Get the dimensions of the output as (width, height).
    pub fn dimensions(&self) -> (i32, i32) {
        unsafe { ((*self.output).width, (*self.output).height) }
    }

    /// Get the physical dimensions of the output as (width, height).
    pub fn physical_dimensions(&self) -> (i32, i32) {
        unsafe { ((*self.output).phys_width, (*self.output).phys_height) }
    }

    pub fn effective_resolution(&self) -> (i32, i32) {
        unsafe {
            let (mut x, mut y) = (0, 0);
            wlr_output_effective_resolution(self.output, &mut x, &mut y);
            (x, y)
        }
    }

    pub fn transform_matrix(&self) -> [f32; 16] {
        unsafe { (*self.output).transform_matrix }
    }

    pub fn transform(&mut self, transform: wl_output_transform) {
        unsafe {
            wlr_output_transform(self.output, transform);
        }
    }

    // TODO Wrap this somehow? Hmm
    pub unsafe fn modes(&self) -> *mut wl_list {
        &mut (*self.output).modes
    }

    pub unsafe fn events(&self) -> wlr_output_events {
        (*self.output).events
    }

    pub unsafe fn from_ptr(output: *mut wlr_output) -> Self {
        OutputHandle { output }
    }

    pub unsafe fn to_ptr(&self) -> *mut wlr_output {
        self.output
    }
}

impl Drop for OutputHandle {
    fn drop(&mut self) {
        // NOTE
        // We do _not_ need to call wlr_output_destroy for the output
        // That is handled by the backend automatically
        // This is left here so no one does this in the future.
    }
}

impl OutputLayout {
    pub fn new() -> Self {
        unsafe { OutputLayout { layout: wlr_output_layout_create() } }
    }

    pub unsafe fn to_ptr(&self) -> *mut wlr_output_layout {
        self.layout
    }

    pub unsafe fn from_ptr(layout: *mut wlr_output_layout) -> Self {
        OutputLayout { layout }
    }

    /// # Unsafety
    /// The underlying function hasn't been proven to be stable if you
    /// pass it an invalid OutputHandle (e.g one that has already been freed).
    /// For now, this function is unsafe
    pub unsafe fn remove(&mut self, output: &mut OutputHandle) {
        wlr_output_layout_remove(self.layout, output.to_ptr())
    }
}

impl Drop for OutputLayout {
    fn drop(&mut self) {
        unsafe { wlr_output_layout_destroy(self.layout) }
    }
}
