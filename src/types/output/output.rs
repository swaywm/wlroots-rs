//! TODO Documentation

use std::{panic, ptr};
use std::ffi::CStr;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::{wl_list, wl_output_transform, wlr_output, wlr_output_effective_resolution,
                  wlr_output_events, wlr_output_make_current, wlr_output_mode,
                  wlr_output_set_mode, wlr_output_set_transform, wlr_output_swap_buffers};

use super::output_layout::OutputLayoutHandle;
use errors::{UpgradeHandleErr, UpgradeHandleResult};

pub(crate) struct OutputState {
    pub layout_handle: OutputLayoutHandle
}

#[derive(Debug)]
pub struct Output {
    /// The structure that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `OutputHandle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `OutputHandle`.
    liveliness: Option<Rc<AtomicBool>>,
    /// The output ptr that refers to this `Output`
    output: *mut wlr_output
}

/// A wrapper around a wlr_output.
#[derive(Debug)]
pub struct OutputHandle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the pointer associated with this handle,
    /// this can no longer be used.
    handle: Weak<AtomicBool>,
    /// The output ptr that refers to this `Output`
    output: *mut wlr_output
}

impl Output {
    /// Just like `std::clone::Clone`, but unsafe.
    ///
    /// # Unsafety
    /// You can create a reference leak using this method very easily,
    /// and it could make it so that weak handles to an output are dangling.
    ///
    /// This exists due to an issue in output_manager.rs that might be fixed
    /// with NLL, so if this is no longer necessary it should be removed asap.
    pub(crate) unsafe fn clone(&self) -> Output {
        Output { liveliness: self.liveliness.clone(),
                 output: self.output }
    }

    /// Makes a new `Output` from a `wlr_output`.
    ///
    /// # Safety
    /// This creates a totally new Output (e.g with its own reference count)
    /// so only do this once per `wlr_output`!
    pub(crate) unsafe fn new(output: *mut wlr_output) -> Self {
        (*output).data = ptr::null_mut();
        Output { liveliness: Some(Rc::new(AtomicBool::new(false))),
                 output }
    }

    pub(crate) unsafe fn set_user_data(&mut self, data: Box<OutputState>) {
        self.remove_from_output_layout();
        (*self.output).data = Box::into_raw(data) as *mut _
    }

    pub(crate) unsafe fn user_data(&mut self) -> *mut OutputState {
        (*self.output).data as *mut _
    }

    /// Used to clear the pointer to an OutputLayout when the OutputLayout
    /// removes this Output from its listing.
    pub(crate) unsafe fn clear_user_data(&mut self) {
        let user_data = self.user_data();
        if user_data.is_null() {
            return
        }
        let _ = Box::from_raw(user_data);
        (*self.output).data = ptr::null_mut();
    }

    /// Remove this Output from an OutputLayout, if it is part of an
    /// OutputLayout.
    pub(crate) unsafe fn remove_from_output_layout(&mut self) {
        if !(*self.output).data.is_null() {
            // Remove output from previous output layout.
            let mut layout_handle = (*self.user_data()).layout_handle.clone();
            match layout_handle.run(|layout| layout.remove(self)) {
                Ok(_) | Err(UpgradeHandleErr::AlreadyDropped) => {}
                Err(UpgradeHandleErr::AlreadyBorrowed) => {
                    panic!("Could not add OutputLayout to Output user data!")
                }
            }
        }
    }

    /// Gets the OutputLayout this Output is a part of, if it is part
    /// of an OutputLayout.
    ///
    /// # Safety
    /// Note that this isn't exposed to user space, as they could easily
    /// create two mutable pointers to the same structure. We keep it internally
    /// though because we use it during the cleanup process.
    pub(crate) unsafe fn layout(&mut self) -> Option<OutputLayoutHandle> {
        let data = self.user_data();
        if data.is_null() {
            None
        } else {
            Some((*data).layout_handle.clone())
        }
    }

    /// Sets the best modesetting for an output.
    ///
    /// NOTE You _cannot_ call this when the output will be removed.
    ///
    /// I'm still marking it as safe though because we protect against that
    /// action in the output destruction callback.
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
                wlr_output_set_mode(self.as_ptr(), first_mode_ptr);
            }
        }
    }

    /// Gets the name of the output in UTF-8.
    pub fn name(&self) -> String {
        unsafe {
            CStr::from_ptr((*self.output).name.as_ptr()).to_string_lossy()
                                                        .into_owned()
        }
    }

    /// Gets the make of the output in UTF-8.
    pub fn make(&self) -> String {
        unsafe {
            CStr::from_ptr((*self.output).make.as_ptr()).to_string_lossy()
                                                        .into_owned()
        }
    }

    /// Gets the model of the output in UTF-8.
    pub fn model(&self) -> String {
        unsafe {
            CStr::from_ptr((*self.output).model.as_ptr()).to_string_lossy()
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
            wlr_output_set_transform(self.output, transform);
        }
    }

    /// TODO Make safe
    pub unsafe fn modes(&self) -> *mut wl_list {
        &mut (*self.output).modes
    }

    /// TODO Make safe
    pub unsafe fn events(&self) -> wlr_output_events {
        (*self.output).events
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_output {
        self.output
    }

    /// Creates a weak reference to an `Output`.
    ///
    /// # Panics
    /// If this `Output` is a previously upgraded `OutputHandle`,
    /// then this function will panic.
    pub fn weak_reference(&self) -> OutputHandle {
        let arc = self.liveliness.as_ref()
                      .expect("Cannot downgrade a previously upgraded OutputHandle");
        OutputHandle { handle: Rc::downgrade(arc),
                       output: self.output }
    }

    unsafe fn from_handle(handle: &OutputHandle) -> Self {
        Output { liveliness: None,
                 output: handle.as_ptr() }
    }

    /// Manually set the lock used to determine if a double-borrow is
    /// occuring on this structure.
    ///
    /// # Panics
    /// Panics when trying to set the lock on an upgraded handle.
    pub(crate) unsafe fn set_lock(&self, val: bool) {
        self.liveliness.as_ref()
            .expect("Tried to set lock on borrowed Output")
            .store(val, Ordering::Release);
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        // NOTE
        // We do _not_ need to call wlr_output_destroy for the output
        // That is handled by the backend automatically
        match self.liveliness {
            None => return,
            Some(ref liveliness) => {
                if Rc::strong_count(liveliness) == 1 {
                    wlr_log!(L_DEBUG, "Dropped output {:p}", self.output);
                    let weak_count = Rc::weak_count(liveliness);
                    if weak_count > 0 {
                        wlr_log!(L_DEBUG,
                                 "Still {} weak pointers to Output {:p}",
                                 weak_count,
                                 self.output);
                    }
                } else {
                    return
                }
            }
        }
        // TODO Move back up in the some after NLL is a thing.
        unsafe {
            self.remove_from_output_layout();
        }
    }
}

impl OutputHandle {
    /// Upgrades the output handle to a reference to the backing `Output`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Output`
    /// which may live forever..
    /// But no output lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> UpgradeHandleResult<Output> {
        self.handle.upgrade()
            .ok_or(UpgradeHandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let output = Output::from_handle(self);
                if check.load(Ordering::Acquire) {
                    return Err(UpgradeHandleErr::AlreadyBorrowed)
                }
                check.store(true, Ordering::Release);
                Ok(output)
            })
    }

    /// Run a function on the referenced Output, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the output
    /// to a short lived scope of an anonymous function,
    /// this function ensures the Output does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Output`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&mut self, runner: F) -> UpgradeHandleResult<Option<R>>
        where F: FnOnce(&mut Output) -> R
    {
        let mut output = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| Some(runner(&mut output))));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.load(Ordering::Acquire) {
                                          wlr_log!(L_ERROR,
                                                   "After running output callback, mutable lock \
                                                    was false for: {:?}",
                                                   output);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.store(false, Ordering::Release);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_output {
        self.output
    }
}
