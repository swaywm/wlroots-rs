//! TODO Documentation

use std::marker::PhantomData;

use wlroots_sys::wlr_output_mode;

use output::Output;

#[derive(Debug, Eq, PartialEq)]
pub struct OutputMode<'output> {
    output_mode: *mut wlr_output_mode,
    phantom: PhantomData<&'output Output>
}

impl<'output> OutputMode<'output> {
    /// NOTE This is a lifetime defined by the user of this function, but it must not outlive
    /// the `Output` that hosts this output mode.
    pub(crate) unsafe fn new<'unbound>(output_mode: *mut wlr_output_mode) -> OutputMode<'unbound> {
        OutputMode { output_mode,
                     phantom: PhantomData }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_output_mode {
        self.output_mode
    }

    /// Gets the flags set on this OutputMode.
    pub fn flags(&self) -> u32 {
        unsafe { (*self.output_mode).flags }
    }

    /// Gets the dimensions of this OutputMode.
    ///
    /// Returned value is (width, height)
    pub fn dimensions(&self) -> (i32, i32) {
        unsafe { ((*self.output_mode).width, (*self.output_mode).height) }
    }

    /// Get the refresh value of the output.
    pub fn refresh(&self) -> i32 {
        unsafe { (*self.output_mode).refresh }
    }
}
