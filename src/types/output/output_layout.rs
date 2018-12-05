//! TODO Documentation

use std::{fmt, panic, ptr, cell::Cell, marker::PhantomData, rc::{Rc, Weak}};

use libc::{self, c_double, c_int};
use wayland_sys::server::{signal::wl_signal_add, WAYLAND_SERVER_HANDLE};
use wlroots_sys::{wlr_output_effective_resolution, wlr_output_layout, wlr_output_layout_add,
                  wlr_output_layout_add_auto, wlr_output_layout_closest_point,
                  wlr_output_layout_contains_point, wlr_output_layout_create,
                  wlr_output_layout_destroy, wlr_output_layout_get, wlr_output_layout_get_box,
                  wlr_output_layout_get_center_output, wlr_output_layout_intersects,
                  wlr_output_layout_move, wlr_output_layout_output, wlr_output_layout_output_at,
                  wlr_output_layout_output_coords, wlr_output_layout_remove};

use {area::{Area, Origin},
     compositor::{compositor_handle, CompositorHandle},
     output::{Output, OutputHandle},
     errors::{HandleErr, HandleResult}};

struct OutputLayoutState {
    /// A counter that will always have a strong count of 1.
    ///
    /// Once the output layout is destroyed, this will signal to the `OutputHandle`s that
    /// they cannot be upgraded.
    counter: Rc<Cell<bool>>,
    /// A raw pointer to the `OutputLayout` on the heap.
    layout: *mut OutputLayout
}

pub trait OutputLayoutHandler {
    /// Callback that's triggered when an output is added to the output layout.
    fn output_added<'this>(&'this mut self,
                           CompositorHandle,
                           OutputLayoutHandle,
                           OutputLayoutOutput<'this>) {
    }

    /// Callback that's triggered when an output is removed from the output
    /// layout.
    fn output_removed<'this>(&'this mut self,
                             CompositorHandle,
                             OutputLayoutHandle,
                             OutputLayoutOutput<'this>) {
    }

    /// Callback that's triggered when the layout changes.
    fn on_change<'this>(&mut self,
                        CompositorHandle,
                        OutputLayoutHandle,
                        OutputLayoutOutput<'this>) {
    }
}

wayland_listener!(pub OutputLayout, (*mut wlr_output_layout, Box<OutputLayoutHandler>), [
    output_add_listener => output_add_notify: |this: &mut OutputLayout, data: *mut libc::c_void,|
    unsafe {
        let (output_ptr, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let layout_output = data as *mut wlr_output_layout_output;
        let layout_output = OutputLayoutOutput{layout_output, phantom: PhantomData};
        let output_layout = OutputLayout::from_ptr(output_ptr);

        manager.output_added(compositor,
                             output_layout.weak_reference(),
                             layout_output);

        Box::into_raw(output_layout);
    };
    output_remove_listener => output_remove_notify: |this: &mut OutputLayout,
                                                     data: *mut libc::c_void,|
    unsafe {
        let (output_ptr, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let layout_output = data as *mut wlr_output_layout_output;
        let layout_output = OutputLayoutOutput { layout_output, phantom: PhantomData};
        let output_layout = OutputLayout::from_ptr(output_ptr);

        manager.output_removed(compositor,
                               output_layout.weak_reference(),
                               layout_output);

        Box::into_raw(output_layout);
    };
    change_listener => change_notify: |this: &mut OutputLayout, data: *mut libc::c_void,|
    unsafe {
        let (output_ptr, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let layout_output = data as *mut wlr_output_layout_output;
        let layout_output = OutputLayoutOutput { layout_output, phantom: PhantomData};
        let output_layout = OutputLayout::from_ptr(output_ptr);

        manager.on_change(compositor,
                          output_layout.weak_reference(),
                          layout_output);

        Box::into_raw(output_layout);
    };
]);

impl fmt::Debug for OutputLayout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

/// A handle to an `OutputLayout`.
#[derive(Debug, Clone)]
pub struct OutputLayoutHandle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the pointer associated with this handle,
    /// this can no longer be used.
    handle: Weak<Cell<bool>>,
    /// The output_layout ptr that refers to this `OutputLayout`
    layout: *mut wlr_output_layout
}

/// The coordinate information of an `Output` within an `OutputLayout`.
#[derive(Debug)]
pub struct OutputLayoutOutput<'output> {
    layout_output: *mut wlr_output_layout_output,
    phantom: PhantomData<&'output OutputLayout>
}

impl OutputLayout {
    /// Construct a new OuputLayout.
    pub fn create(handler: Box<OutputLayoutHandler>) -> OutputLayoutHandle {
        unsafe {
            let layout = wlr_output_layout_create();
            if layout.is_null() {
                panic!("Could not allocate a wlr_output_layout")
            }
            let mut output_layout = OutputLayout::new((layout, handler));
            wl_signal_add(&mut (*layout).events.add as *mut _ as _,
                          output_layout.output_add_listener() as *mut _ as _);
            wl_signal_add(&mut (*layout).events.destroy as *mut _ as _,
                          output_layout.output_remove_listener() as *mut _ as _);
            wl_signal_add(&mut (*layout).events.change as *mut _ as _,
                          output_layout.change_listener() as *mut _ as _);
            let counter = Rc::new(Cell::new(false));
            let handle = Rc::downgrade(&counter);
            let state = Box::new(OutputLayoutState { counter,
                                                     layout: Box::into_raw(output_layout) });
            (*layout).data = Box::into_raw(state) as *mut libc::c_void;
            OutputLayoutHandle { layout, handle }
        }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_output_layout {
        self.data.0
    }

    /// Reconstruct the box from the wlr_output_layout.
    unsafe fn from_ptr(layout: *mut wlr_output_layout) -> Box<OutputLayout> {
        let data = (*layout).data as *mut OutputLayoutState;
        if data.is_null() {
            panic!("Data pointer on the output layout was null!");
        }
        Box::from_raw((*data).layout)
    }

    /// Get the outputs associated with this OutputLayout.
    ///
    /// Also returns their absolute position within the layout.
    pub fn outputs(&mut self) -> Vec<(OutputHandle, Origin)> {
        unsafe {
            let mut result = vec![];
            wl_list_for_each!((*self.data.0).outputs, link, (pos: wlr_output_layout_output) => {
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
            wl_list_for_each!((*self.data.0).outputs, link,
                              (layout_output: wlr_output_layout_output) => {
                                  result.push(OutputLayoutOutput { layout_output,
                                                                   phantom: PhantomData
                                  })
                              });
            result
        }
    }

    /// Adds an output to the layout at the given coordinates.
    pub fn add(&mut self, output: &mut Output, origin: Origin) {
        let (x, y) = (origin.x, origin.y);
        unsafe { wlr_output_layout_add(self.data.0, output.as_ptr(), x, y) }
    }

    /// Adds an output to the layout, automatically positioning it with
    /// the others that are already there.
    pub fn add_auto(&mut self, output: &mut Output) {
        unsafe {
            let layout_handle = self.weak_reference();
            output.set_output_layout(Some(layout_handle));
            wlr_output_layout_add_auto(self.data.0, output.as_ptr());
            wlr_log!(WLR_DEBUG, "Added {:?} to {:?}", output, self);
        }
    }

    /// Moves the output to the given coordinates.
    ///
    /// If the output is not part of this layout this does nothing.
    pub fn move_output(&mut self, output: &mut Output, origin: Origin) {
        let (x, y) = (origin.x, origin.y);
        unsafe { wlr_output_layout_move(self.data.0, output.as_ptr(), x, y) }
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
            wlr_output_layout_closest_point(self.data.0, output_ptr, x, y, out_x, out_y);
            (*out_x, *out_y)
        }
    }

    /// Determines if the `OutputLayout` contains the `Output` at the given
    /// point.
    pub fn contains_point(&mut self, output: &mut Output, origin: Origin) -> bool {
        unsafe {
            wlr_output_layout_contains_point(self.data.0, output.as_ptr(), origin.x, origin.y)
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
            Area::from_box(*wlr_output_layout_get_box(self.data.0, output_ptr))
        }
    }

    /// Get the output closest to the center of the layout extents, if one
    /// exists.
    pub fn get_center_output(&mut self) -> Option<OutputHandle> {
        unsafe {
            let output = wlr_output_layout_get_center_output(self.data.0);
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
        unsafe { wlr_output_layout_intersects(self.data.0, output.as_ptr(), &area.into()) }
    }

    /// Given x and y as pointers to global coordinates, adjusts them to local output
    /// coordinates relative to the given reference output.
    pub fn output_coords(&mut self, output: &mut Output, x: &mut f64, y: &mut f64) {
        unsafe { wlr_output_layout_output_coords(self.data.0, output.as_ptr(), x, y) }
    }

    /// Remove an output from this layout.
    ///
    /// If the output was not in the layout, does nothing.
    pub fn remove(&mut self, output: &mut Output) {
        wlr_log!(WLR_DEBUG, "Removing {:?} from {:?}", output, self);
        unsafe {
            output.clear_output_layout_data();
            wlr_output_layout_remove(self.data.0, output.as_ptr());
        };
    }

    /// Get an output's information about its place in the `OutputLayout`, if
    /// it's present.
    pub fn get_output_info<'output>(&mut self,
                                    output: &'output mut Output)
                                    -> Option<OutputLayoutOutput<'output>> {
        unsafe {
            let layout_output = wlr_output_layout_get(self.data.0, output.as_ptr());
            if layout_output.is_null() {
                None
            } else {
                Some(OutputLayoutOutput { layout_output,
                                          phantom: PhantomData })
            }
        }
    }

    /// Get the output at the given output layout coordinate location, if there
    /// is one there.
    pub fn output_at(&mut self, lx: c_double, ly: c_double) -> Option<OutputHandle> {
        unsafe {
            let output = wlr_output_layout_output_at(self.data.0, lx, ly);
            if output.is_null() {
                None
            } else {
                Some(OutputHandle::from_ptr(output))
            }
        }
    }

    /// Creates a weak reference to an `OutputLayout`.
    ///
    /// # Panics
    /// If this `OutputLayout` is a previously upgraded `OutputLayoutHandle`,
    /// then this function will panic.
    pub fn weak_reference(&self) -> OutputLayoutHandle {
        unsafe {
            let handle = Rc::downgrade(&(*((*self.data.0).data as *mut OutputLayoutState)).counter);
            OutputLayoutHandle { layout: self.data.0,
                                 handle }
        }
    }
}

impl Drop for OutputLayout {
    fn drop(&mut self) {
        let layout_ptr = self.data.0;
        unsafe {
            let data = Box::from_raw((*layout_ptr).data as *mut OutputLayoutState);
            let mut manager = Box::from_raw(data.layout);
            assert_eq!(Rc::strong_count(&data.counter),
                       1,
                       "OutputLayout had more than 1 reference count");
            (*layout_ptr).data = ptr::null_mut();
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*manager.output_add_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*manager.output_remove_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*manager.change_listener()).link as *mut _ as _);
            wlr_output_layout_destroy(self.data.0)
        }
    }
}

impl OutputLayoutHandle {
    /// Constructs a new OutputLayoutHandle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            OutputLayoutHandle { handle: Weak::new(),
                                 layout: ptr::null_mut() }
        }
    }

    /// Upgrades the `OutputLayoutHandle` to a reference
    /// to the backing `OutputLayout`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `OutputLayout`
    /// which may live forever..
    /// But the actual lifetime of `OutputLayout` is determined by the user.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<Box<OutputLayout>> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.set(true);
                Ok(OutputLayout::from_ptr(self.layout))
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
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut OutputLayout) -> R
    {
        let mut output_layout = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut output_layout)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(WLR_ERROR,
                                                   "After running OutputLayout callback, mutable \
                                                    lock was false for: {:?}",
                                                   output_layout);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.set(false);
                                  });
        Box::into_raw(output_layout);
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    /// Destroy the output layout that this handle refers to.
    ///
    /// This will invalidate the other handles, including the ones held by
    /// `Output`s.
    ///
    /// If the output layout was previously destroyed, does nothing.
    pub fn destroy(self) {
        unsafe {
            self.upgrade().ok();
        }
    }
}

impl<'output> OutputLayoutOutput<'output> {
    /// Get a handle to the output that this structure describes.
    pub fn output(&self) -> OutputHandle {
        unsafe { OutputHandle::from_ptr((*self.layout_output).output) }
    }

    /// Get the coordinates of this output in the layout output.
    pub fn coords(&self) -> (c_int, c_int) {
        unsafe { ((*self.layout_output).x, (*self.layout_output).y) }
    }

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

impl Default for OutputLayoutHandle {
    fn default() -> Self {
        OutputLayoutHandle::new()
    }
}

impl PartialEq for OutputLayoutHandle {
    fn eq(&self, other: &OutputLayoutHandle) -> bool {
        self.layout == other.layout
    }
}

impl Eq for OutputLayoutHandle {}
