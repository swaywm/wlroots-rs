use std::ffi::CStr;
use wlroots_sys::{list_t, wlr_output, wlr_output__bindgen_ty_1};

/// A wrapper around a wlr_output.
pub struct Output {
    output: *mut wlr_output
}

// TODO We are assuming the output is live in these functions,
// but we need some way to ensure that.
// E.g we need to control access to the "Output",
// probably only in certain methods.

impl Output {
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

    /// Get the dimensions of the output as (width, height).
    pub fn dimensions(&self) -> (i32, i32) {
        unsafe { ((*self.output).width, (*self.output).height) }
    }

    /// Get the physical dimensions of the output as (width, height).
    pub fn physical_dimensions(&self) -> (i32, i32) {
        unsafe { ((*self.output).phys_width, (*self.output).phys_height) }
    }

    // TODO Wrap this somehow? Hmm
    pub unsafe fn modes(&self) -> *mut list_t {
        (*self.output).modes
    }

    // FIXME Really need to change the name of this type
    pub unsafe fn events(&self) -> wlr_output__bindgen_ty_1 {
        (*self.output).events
    }

    pub unsafe fn from_ptr(output: *mut wlr_output) -> Self {
        Output { output }
    }

    pub unsafe fn to_ptr(&self) -> *mut wlr_output {
        self.output
    }
}
