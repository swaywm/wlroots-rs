//! TODO Documentation

use std::panic;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use wlroots_sys::{wlr_cursor_attach_output_layout, wlr_output_effective_resolution,
                  wlr_output_layout, wlr_output_layout_add, wlr_output_layout_add_auto,
                  wlr_output_layout_create, wlr_output_layout_destroy, wlr_output_layout_move,
                  wlr_output_layout_output, wlr_output_layout_remove};

use super::output::OutputState;
use errors::{UpgradeHandleErr, UpgradeHandleResult};

use {Cursor, CursorBuilder, Origin, Output, OutputHandle};

#[derive(Debug)]
pub struct OutputLayout {
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
    /// The output_layout ptr that refers to this `OutputLayout`
    layout: *mut wlr_output_layout,
    cursors: Vec<Cursor>
}

/// A handle to an `OutputLayout`.
///
/// Used internally by `Output` to gain access to the `OutputLayout`.
#[derive(Debug, Clone)]
pub(crate) struct OutputLayoutHandle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the pointer associated with this handle,
    /// this can no longer be used.
    handle: Weak<AtomicBool>,
    /// The output_layout ptr that refers to this `OutputLayout`
    layout: *mut wlr_output_layout
}

/// The coordinate information of an `Output` within an `OutputLayout`.
#[derive(Debug)]
// TODO Remove pub?
pub struct OutputLayoutOutput {
    layout_output: *mut wlr_output_layout_output
}

impl OutputLayout {
    /// Construct a new OuputLayout.
    pub fn new() -> Option<Self> {
        unsafe {
            let layout = wlr_output_layout_create();
            if layout.is_null() {
                None
            } else {
                Some(OutputLayout { liveliness: Some(Rc::new(AtomicBool::new(false))),
                                    layout,
                                    cursors: Vec::new() })
            }
        }
    }

    pub fn outputs(&mut self) -> Vec<(OutputHandle, Origin)> {
        unsafe {
            let mut result = vec![];
            // TODO Macro for for-each pattern here
            let mut pos = container_of!((*self.layout).outputs.next, wlr_output_layout_output, link);
            loop {
                if &(*pos).link as *const _ == &(*self.layout).outputs as *const _ {
                    return result;
                }
                // TODO Get details from the user data
                result.push((Output::new((*pos).output).weak_reference(),
                             Origin::new((*pos).x, (*pos).y)));
                pos = container_of!((*pos).link.next, wlr_output_layout_output, link);
            }
        }
    }

    pub fn cursors(&mut self) -> &mut [Cursor] {
        self.cursors.as_mut_slice()
    }

    /// Attach a cursor to this OutputLayout.
    pub fn attach_cursor(&mut self, cursor: CursorBuilder) {
        unsafe {
            let cursor = cursor.build(self.weak_reference());
            wlr_cursor_attach_output_layout(cursor.as_ptr(), self.layout);
            self.cursors.push(cursor);
        }
    }

    /// Adds an output to the layout at the given coordinates.
    pub fn add(&mut self, output: &mut Output, origin: Origin) {
        let (x, y) = (origin.x, origin.y);
        unsafe { wlr_output_layout_add(self.layout, output.as_ptr(), x, y) }
    }

    /// Adds an output to the layout, automatically positioning it with
    /// the others that are already there.
    pub fn add_auto(&mut self, output: &mut Output) {
        unsafe {
            let layout_handle = self.weak_reference();
            output.set_user_data(Box::new(OutputState { layout_handle }));
            wlr_output_layout_add_auto(self.layout, output.as_ptr());
            wlr_log!(L_DEBUG, "Added {:?} to {:?}", output, self);
        }
    }

    /// Moves the output to the given coordinates.
    ///
    /// If the output is not part of this layout this does nothing.
    pub fn move_output(&mut self, output: &mut Output, origin: Origin) {
        let (x, y) = (origin.x, origin.y);
        unsafe { wlr_output_layout_move(self.layout, output.as_ptr(), x, y) }
    }

    /// Remove an output from this layout.
    ///
    /// If the output was not in the layout, does nothing.
    pub fn remove(&mut self, output: &mut Output) {
        wlr_log!(L_DEBUG, "Removing {:?} from {:?}", output, self);
        unsafe {
            output.clear_user_data();
            wlr_output_layout_remove(self.layout, output.as_ptr());
        };
    }

    /// Creates a weak reference to an `OutputLayout`.
    ///
    /// # Panics
    /// If this `OutputLayout` is a previously upgraded `OutputLayoutHandle`,
    /// then this function will panic.
    pub(crate) fn weak_reference(&self) -> OutputLayoutHandle {
        let arc = self.liveliness.as_ref()
                      .expect("Cannot downgrade a previously upgraded OutputLayoutHandle");
        OutputLayoutHandle { handle: Rc::downgrade(arc),
                             layout: self.layout }
    }

    unsafe fn from_handle(handle: &OutputLayoutHandle) -> Self {
        OutputLayout { liveliness: None,
                       layout: handle.as_ptr(),
                       cursors: Vec::new() }
    }
}

impl Drop for OutputLayout {
    fn drop(&mut self) {
        match self.liveliness {
            None => {}
            Some(ref liveliness) => {
                if Rc::strong_count(liveliness) == 1 {
                    unsafe { wlr_output_layout_destroy(self.layout) }
                    wlr_log!(L_DEBUG, "Dropped {:?}", self);
                    let weak_count = Rc::weak_count(liveliness);
                    if weak_count > 0 {
                        wlr_log!(L_DEBUG,
                                 "Still {} weak pointers to OutputLayout {:?}",
                                 weak_count,
                                 self.layout);
                    }
                }
            }
        }
    }
}

impl OutputLayoutHandle {
    /// Upgrades the `OutputLayoutHandle` to a reference
    /// to the backing `OutputLayout`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `OutputLayout`
    /// which may live forever..
    /// But the actual lifetime of `OutputLayout` is determined by the user.
    pub(crate) unsafe fn upgrade(&self) -> UpgradeHandleResult<OutputLayout> {
        self.handle.upgrade()
            .ok_or(UpgradeHandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let output_layout = OutputLayout::from_handle(self);
                if check.load(Ordering::Acquire) {
                    return Err(UpgradeHandleErr::AlreadyBorrowed)
                }
                check.store(true, Ordering::Release);
                Ok(output_layout)
            })
    }

    /// Run a function on the referenced OutputLayout, if it still exists
    ///
    /// Returns the result of the function, if successful.
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the OutputLayout
    /// to a short lived scope of an anonymous function,
    /// this function ensures the OutputLayout does not live longer
    /// than it exists (because the lifetime is controlled by the user).
    pub fn run<F, R>(&mut self, runner: F) -> UpgradeHandleResult<R>
        where F: FnOnce(&mut OutputLayout) -> R
    {
        let mut output_layout = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut output_layout)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.load(Ordering::Acquire) {
                                          wlr_log!(L_ERROR,
                                                   "After running OutputLayout callback, mutable \
                                                    lock was false for: {:?}",
                                                   output_layout);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.store(false, Ordering::Release);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    unsafe fn as_ptr(&self) -> *mut wlr_output_layout {
        self.layout
    }
}

impl OutputLayoutOutput {
    /// Get the absolute top left edge coordinate of this output in the output
    /// layout.
    pub fn top_left_edge(&self) -> Origin {
        unsafe { Origin::new((*self.layout_output).x, (*self.layout_output).y) }
    }

    /// Get the absolute top right edge coordinate of this output in the output
    /// layout.
    pub fn top_right_edge(&self) -> Origin {
        unsafe {
            let (mut width, mut _height) = (0, 0);
            wlr_output_effective_resolution((*self.layout_output).output, &mut width, &mut _height);
            let (x, y) = ((*self.layout_output).x, (*self.layout_output).y);
            Origin::new(x + width, y)
        }
    }

    pub fn bottom_left_edge(&self) -> Origin {
        unsafe {
            let (mut _width, mut height) = (0, 0);
            wlr_output_effective_resolution((*self.layout_output).output, &mut _width, &mut height);
            let (x, y) = ((*self.layout_output).x, (*self.layout_output).y);
            Origin::new(x, y + height)
        }
    }

    pub fn bottom_right_edge(&self) -> Origin {
        unsafe {
            let (mut width, mut height) = (0, 0);
            wlr_output_effective_resolution((*self.layout_output).output, &mut width, &mut height);
            let (x, y) = ((*self.layout_output).x, (*self.layout_output).y);
            Origin::new(x + height, y + height)
        }
    }
}

impl PartialEq for OutputLayoutHandle {
    fn eq(&self, other: &OutputLayoutHandle) -> bool {
        self.layout == other.layout
    }
}

impl Eq for OutputLayoutHandle {}
