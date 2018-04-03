//! TODO Documentation

use std::{panic, ptr};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use wlroots_sys::{wlr_cursor_attach_output_layout, wlr_output_effective_resolution,
                  wlr_output_layout, wlr_output_layout_add, wlr_output_layout_add_auto,
                  wlr_output_layout_closest_point, wlr_output_layout_contains_point,
                  wlr_output_layout_create, wlr_output_layout_destroy, wlr_output_layout_get_box,
                  wlr_output_layout_get_center_output, wlr_output_layout_intersects,
                  wlr_output_layout_move, wlr_output_layout_output,
                  wlr_output_layout_output_coords, wlr_output_layout_remove};

use errors::{UpgradeHandleErr, UpgradeHandleResult};

use {Area, Cursor, CursorBuilder, CursorId, CursorWrapper, Origin, Output, OutputHandle};

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
    /// The cursors attached to this output layout.
    pub(crate) cursors: Rc<RefCell<HashMap<CursorId, Box<CursorWrapper>>>>
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
    layout: *mut wlr_output_layout,
    /// The cursors attached to this output layout.
    cursors: Weak<RefCell<HashMap<CursorId, Box<CursorWrapper>>>>
}

/// The coordinate information of an `Output` within an `OutputLayout`.
#[derive(Debug)]
pub struct OutputLayoutOutput<'output> {
    layout_output: *mut wlr_output_layout_output,
    phantom: PhantomData<&'output OutputLayout>
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
                                    cursors: Rc::new(RefCell::new(HashMap::new())) })
            }
        }
    }

    /// Get the outputs associated with this OutputLayout.
    ///
    /// Also returns their absolute position within the layout.
    pub fn outputs(&mut self) -> Vec<(OutputHandle, Origin)> {
        unsafe {
            let mut result = vec![];
            wl_list_for_each!((*self.layout).outputs, link, (pos: wlr_output_layout_output) => {
                result.push((OutputHandle::from_ptr((*pos).output),
                             Origin::new((*pos).x, (*pos).y)))
            });
            result
        }
    }

    /// Get the Outputs in the OutputLayout coupled with their output information.
    ///
    /// For a version that isn't bound by lifetimes, see `outputs`.
    pub fn outputs_layouts<'output>(&'output mut self) -> Vec<OutputLayoutOutput<'output>> {
        unsafe {
            let mut result = vec![];
            wl_list_for_each!((*self.layout).outputs, link,
                              (layout_output: wlr_output_layout_output) => {
                                  result.push(OutputLayoutOutput { layout_output,
                                                                   phantom: PhantomData
                                  })
                              });
            result
        }
    }

    /// Gets a cursor by its id.
    ///
    /// If the cursor has been dropped, or is currently being borrowed, `None` is returned.
    pub fn cursor(&mut self, id: CursorId) -> Option<RefMut<Cursor>> {
        let borrow = self.cursors.borrow_mut();
        if borrow.get(&id).is_none() {
            return None
        }
        Some(RefMut::map(borrow, |borrow| {
                 borrow.get_mut(&id).map(|boxed| boxed.cursor()).unwrap()
             }))
    }

    /// Drops the cursor associated with the id.
    ///
    /// **Note that if the Cursor is being borrowed in a callback, it will not be dropped**.
    pub fn drop_cursor(&mut self, id: CursorId) {
        self.cursors.borrow_mut().remove(&id);
    }

    /// Apply some operations to all cursors.
    pub fn apply_to_cursors<F>(&mut self, mut f: F)
        where F: FnMut(&mut Cursor) -> ()
    {
        let mut borrow = self.cursors.borrow_mut();
        for cursor in borrow.values_mut().map(|boxed| boxed.cursor()) {
            f(cursor)
        }
    }

    /// Attach a cursor to this OutputLayout.
    ///
    /// Returns the id of the cursor so it
    /// can be accessed later.
    pub fn attach_cursor(&mut self, cursor: CursorBuilder) -> CursorId {
        unsafe {
            let cursor = cursor.build(self.weak_reference());
            wlr_cursor_attach_output_layout(cursor.as_ptr(), self.layout);
            let id = CursorId::new(cursor.as_ptr());
            self.cursors.borrow_mut().insert(id, cursor);
            id
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
            output.set_output_layout(Some(layout_handle));
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

    /// Get the closest point on this layout from the given point from the reference
    /// output.
    ///
    /// If reference is None, gets the closest point from the entire layout.
    ///
    /// Returns the closest point in the format (x, y)
    pub fn closest_point<'this, O>(&mut self, reference: O, x: f64, y: f64) -> (f64, f64)
        where O: Into<Option<&'this mut Output>>
    {
        unsafe {
            let output_ptr = reference.into()
                                      .map(|output| output.as_ptr())
                                      .unwrap_or(ptr::null_mut());
            let (ref mut out_x, ref mut out_y) = (0.0, 0.0);
            wlr_output_layout_closest_point(self.layout, output_ptr, x, y, out_x, out_y);
            (*out_x, *out_y)
        }
    }

    /// Determines if the `OutputLayout` contains the `Output` at the given
    /// point.
    pub fn contains_point(&mut self, output: &mut Output, origin: Origin) -> bool {
        unsafe {
            wlr_output_layout_contains_point(self.layout, output.as_ptr(), origin.x, origin.y)
        }
    }

    /// Get the box of the layout for the given reference output.
    ///
    /// If `reference` is None, the box will be for the extents of the entire layout.
    pub fn get_box<'this, O>(&mut self, reference: O) -> Area
        where O: Into<Option<&'this mut Output>>
    {
        unsafe {
            let output_ptr = reference.into()
                                      .map(|output| output.as_ptr())
                                      .unwrap_or(ptr::null_mut());
            Area::from_box(*wlr_output_layout_get_box(self.layout, output_ptr))
        }
    }

    /// Get the output closest to the center of the layout extents, if one
    /// exists.
    pub fn get_center_output(&mut self) -> Option<OutputHandle> {
        unsafe {
            let output = wlr_output_layout_get_center_output(self.layout);
            if output.is_null() {
                None
            } else {
                Some(OutputHandle::from_ptr(output))
            }
        }
    }

    /// Determines if the `Output` in the `OutputLayout` intersects with
    /// the provided `Area`.
    pub fn intersects(&mut self, output: &mut Output, area: Area) -> bool {
        unsafe { wlr_output_layout_intersects(self.layout, output.as_ptr(), &area.into()) }
    }

    /// Given x and y as pointers to global coordinates, adjusts them to local output
    /// coordinates relative to the given reference output.
    pub fn output_coords(&mut self, output: &mut Output, x: &mut f64, y: &mut f64) {
        unsafe { wlr_output_layout_output_coords(self.layout, output.as_ptr(), x, y) }
    }

    /// Remove an output from this layout.
    ///
    /// If the output was not in the layout, does nothing.
    pub fn remove(&mut self, output: &mut Output) {
        wlr_log!(L_DEBUG, "Removing {:?} from {:?}", output, self);
        unsafe {
            output.clear_output_layout_data();
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
                             layout: self.layout,
                             cursors: Rc::downgrade(&self.cursors) }
    }

    unsafe fn from_handle(handle: &OutputLayoutHandle) -> Self {
        OutputLayout { liveliness: None,
                       layout: handle.as_ptr(),
                       cursors: handle.cursors
                                      .upgrade()
                                      .expect("Could not upgrade cursors Rc") }
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

impl<'output> OutputLayoutOutput<'output> {
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
