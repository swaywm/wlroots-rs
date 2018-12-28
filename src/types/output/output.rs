//! TODO Documentation

use std::{cell::Cell, ffi::CStr, mem::ManuallyDrop, rc::{Rc, Weak},
          time::Duration, panic, ptr};

use libc::{c_float, c_int, clock_t};
use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::{timespec, wl_list, wl_output_subpixel, wl_output_transform, wlr_output,
                  wlr_output_damage, wlr_output_effective_resolution, wlr_output_enable,
                  wlr_output_get_gamma_size, wlr_output_make_current, wlr_output_mode,
                  wlr_output_schedule_frame, wlr_output_set_custom_mode,
                  wlr_output_set_gamma, wlr_output_set_mode,
                  wlr_output_set_position, wlr_output_set_scale, wlr_output_set_transform,
                  wlr_output_swap_buffers, wlr_output_transformed_resolution};

use {area::{Origin, Size},
     utils::{self, HandleErr, HandleResult, Handleable, c_to_rust_string},
     output::{self, layout},
     render::PixmanRegion};
pub use manager::output_handler::*;
pub use manager::output_manager::{OutputBuilder as Builder, BuilderResult};
pub(crate) use manager::output_manager::Manager;

pub type Subpixel = wl_output_subpixel;
pub type Transform = wl_output_transform;

pub(crate) struct OutputState {
    pub(crate) output: *mut UserOutput,
    handle: Weak<Cell<bool>>,
    damage: *mut wlr_output_damage,
    layout_handle: Option<layout::Handle>
}

#[derive(Debug)]
pub struct Output {
    /// The structure that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `output::Handle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `output::Handle`.
    liveliness: Rc<Cell<bool>>,
    /// The tracker for damage on the output.
    damage: ManuallyDrop<output::Damage>,
    /// The output ptr that refers to this `Output`
    output: *mut wlr_output
}

pub type Handle = utils::Handle<*mut wlr_output_damage, wlr_output, Output>;

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
                 damage: ManuallyDrop::new(self.damage.clone()),
                 output: self.output }
    }

    /// Makes a new `Output` from a `wlr_output`.
    ///
    /// # Safety
    /// This creates a totally new Output (e.g with its own reference count)
    /// so only do this once per `wlr_output`!
    pub(crate) unsafe fn new(output: *mut wlr_output) -> Self {
        (*output).data = ptr::null_mut();
        let liveliness = Rc::new(Cell::new(false));
        let handle = Rc::downgrade(&liveliness);
        let damage = ManuallyDrop::new(output::Damage::new(output));
        let state = Box::new(OutputState { output: ptr::null_mut(),
                                           handle,
                                           damage: damage.as_ptr(),
                                           layout_handle: None });
        (*output).data = Box::into_raw(state) as *mut _;
        Output { liveliness,
                 damage,
                 output }
    }

    pub(crate) unsafe fn set_output_layout<T>(&mut self, layout_handle: T)
        where T: Into<Option<layout::Handle>>
    {
        self.remove_from_output_layout();
        let user_data = self.user_data();
        if user_data.is_null() {
            return
        }
        let mut data = Box::from_raw(user_data);
        data.layout_handle = layout_handle.into();
        (*self.output).data = Box::into_raw(data) as *mut _;
    }

    unsafe fn user_data(&mut self) -> *mut OutputState {
        (*self.output).data as *mut _
    }

    /// Used to clear the pointer to an OutputLayout when the OutputLayout
    /// removes this Output from its listing.
    pub(crate) unsafe fn clear_output_layout_data(&mut self) {
        let user_data = self.user_data();
        if user_data.is_null() {
            return
        }
        let mut data = Box::from_raw(user_data);
        data.layout_handle = None;
        (*self.output).data = Box::into_raw(data) as *mut _;
    }

    /// Remove this Output from an OutputLayout, if it is part of an
    /// OutputLayout.
    pub(crate) unsafe fn remove_from_output_layout(&mut self) {
        let output_data = self.user_data();
        if output_data.is_null() {
            return
        }
        // Remove output from previous output layout.
        if let Some(layout_handle) = (*output_data).layout_handle.take() {
            match layout_handle.run(|layout| layout.remove(self)) {
                Ok(_) | Err(HandleErr::AlreadyDropped) => self.clear_output_layout_data(),
                Err(HandleErr::AlreadyBorrowed) => {
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
    pub(crate) unsafe fn layout(&mut self) -> Option<layout::Handle> {
        let data = self.user_data();
        if data.is_null() {
            None
        } else {
            (*data).layout_handle.clone()
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
            let modes = &mut (*self.output).modes as *mut wl_list;
            let length = ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_list_length, modes as _);
            if length > 0 {
                // TODO Better logging
                wlr_log!(WLR_DEBUG, "output added {:?}", self);
                let first_mode_ptr: *mut wlr_output_mode;
                first_mode_ptr =
                    container_of!(&mut (*(*modes).prev) as *mut _, wlr_output_mode, link);
                wlr_output_set_mode(self.as_ptr(), first_mode_ptr);
            }
        }
    }

    // TODO Could we pass an output mode from the wrong output here?
    // What will happen?

    /// Set this to be the current mode for the Output.
    pub fn set_mode(&mut self, mode: output::Mode) -> bool {
        unsafe { wlr_output_set_mode(self.output, mode.as_ptr()) }
    }

    /// Set a custom mode for this output.
    pub fn set_custom_mode(&mut self, size: Size, refresh: i32) -> bool {
        unsafe { wlr_output_set_custom_mode(self.output, size.width, size.height, refresh) }
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
            c_to_rust_string((*self.output).make.as_ptr()).expect("Could not parse make as UTF-8")
        }
    }

    /// Gets the model of the output in UTF-8.
    pub fn model(&self) -> String {
        unsafe {
            c_to_rust_string((*self.output).model.as_ptr()).expect("Could not parse model as UTF-8")
        }
    }

    /// Gets the serial of the output in UTF-8.
    pub fn serial(&self) -> String {
        unsafe {
            c_to_rust_string((*self.output).serial.as_ptr()).expect("Could not parse serial as \
                                                                     UTF-8")
        }
    }

    /// Determines if the output is enabled or not.
    pub fn enabled(&self) -> bool {
        unsafe { (*self.output).enabled }
    }

    /// Get the scale of the output
    pub fn scale(&self) -> c_float {
        unsafe { (*self.output).scale }
    }

    /// Determines if the output should have its buffers swapped or not.
    pub fn needs_swap(&self) -> bool {
        unsafe { (*self.output).needs_swap }
    }

    /// Get the refresh rate of the output.
    pub fn refresh_rate(&self) -> i32 {
        unsafe { (*self.output).refresh }
    }

    pub fn current_mode<'output>(&'output self) -> Option<output::Mode<'output>> {
        unsafe {
            if (*self.output).current_mode.is_null() {
                None
            } else {
                Some(output::Mode::new((*self.output).current_mode))
            }
        }
    }

    /// Gets the output position in layout space reported to clients.
    pub fn layout_space_pos(&self) -> (i32, i32) {
        unsafe { ((*self.output).lx, (*self.output).ly) }
    }

    /// Get subpixel information about the output.
    pub fn subpixel(&self) -> Subpixel {
        unsafe { (*self.output).subpixel }
    }

    /// Get the transform information about the output.
    pub fn get_transform(&self) -> Transform {
        unsafe { (*self.output).transform }
    }

    /// Manually schedules a `frame` event.
    ///
    /// If a `frame` event is already pending, it is a no-op.
    pub fn schedule_frame(&mut self) {
        unsafe { wlr_output_schedule_frame(self.output) }
    }

    /// Make this output the current output.
    ///
    /// # Unsafety
    /// This is done for rendering purposes, and you should really use
    /// a `GenericRenderer` instead in order to do this.
    ///
    /// Sometimes however you need to do e.g opengl rendering and we haven't
    /// wrapped that. If that's the case, call this first and then swap the buffers.
    ///
    /// Returns the drawing buffer age in number of frames in number of frames,
    /// or None if unknown. This is useful for damage tracking.
    pub unsafe fn make_current(&mut self) -> (bool, Option<c_int>) {
        let mut buffer_age = -1;
        let res = wlr_output_make_current(self.output, &mut buffer_age);
        let buffer_age = if buffer_age == -1 {
            None
        } else {
            Some(buffer_age)
        };
        (res, buffer_age)
    }

    /// Swaps the buffers and draws whatever is in the back buffer on the screen.
    ///
    /// If the time of the frame is not known, set `when` to None.
    ///
    /// If the compositor does not support damage tracking, set `damage` to `None`
    ///
    /// # Unsafety
    /// This is done for rendering purposes, but if called multiple times then
    /// you could cause a deadlock.
    ///
    /// You should try to use a `GenericRenderer`, but sometimes it's necessary to
    /// do your own manual rendering in a compositor. In that case, call `make_current`,
    /// do your rendering, and then call this function.
    pub unsafe fn swap_buffers<'a, T, U>(&mut self, when: T, damage: U) -> bool
        where T: Into<Option<Duration>>,
              U: Into<Option<&'a mut PixmanRegion>>
    {
        let when = when.into().map(|duration| {
                                       timespec { tv_sec: duration.as_secs() as clock_t,
                                                  tv_nsec: duration.subsec_nanos() as clock_t }
                                   });
        let when_ptr =
            when.map(|mut duration| &mut duration as *mut _).unwrap_or_else(|| ptr::null_mut());
        let damage = match damage.into() {
            Some(region) => &mut region.region as *mut _,
            None => ptr::null_mut()
        };
        wlr_output_swap_buffers(self.output, when_ptr, damage)
    }

    /// Determines if a frame is pending or not.
    pub fn frame_pending(&self) -> bool {
        unsafe { (*self.output).frame_pending }
    }

    /// Get the dimensions of the output as (width, height).
    pub fn size(&self) -> (i32, i32) {
        unsafe { ((*self.output).width, (*self.output).height) }
    }

    /// Get the physical dimensions of the output as (width, height).
    pub fn physical_size(&self) -> (i32, i32) {
        unsafe { ((*self.output).phys_width, (*self.output).phys_height) }
    }

    /// Computes the transformed output resolution
    pub fn transformed_resolution(&self) -> (c_int, c_int) {
        unsafe {
            let (mut x, mut y) = (0, 0);
            wlr_output_transformed_resolution(self.output, &mut x, &mut y);
            (x, y)
        }
    }

    /// Computes the transformed and scaled output resolution.
    pub fn effective_resolution(&self) -> (c_int, c_int) {
        unsafe {
            let (mut x, mut y) = (0, 0);
            wlr_output_effective_resolution(self.output, &mut x, &mut y);
            (x, y)
        }
    }

    pub fn transform_matrix(&self) -> [c_float; 9] {
        unsafe { (*self.output).transform_matrix }
    }

    pub fn transform(&mut self, transform: Transform) {
        unsafe {
            wlr_output_set_transform(self.output, transform);
        }
    }

    /// Get the modes associated with this output.
    ///
    /// Note that some backends may have zero modes.
    pub fn modes<'output>(&'output self) -> Vec<output::Mode<'output>> {
        unsafe {
            let mut result = vec![];
            wl_list_for_each!((*self.output).modes, link, (mode: wlr_output_mode) => {
                result.push(output::Mode::new(mode))
            });
            result
        }
    }

    /// Enables or disables an output.
    pub fn enable(&mut self, enable: bool) -> bool {
        unsafe { wlr_output_enable(self.output, enable) }
    }

    /// Sets the gamma based on the size.
    pub fn set_gamma(&mut self, size: usize, mut r: u16, mut g: u16, mut b: u16) -> bool {
        unsafe { wlr_output_set_gamma(self.output, size, &mut r, &mut g, &mut b) }
    }

    /// Get the gamma size.
    pub fn get_gamma_size(&self) -> usize {
        unsafe { wlr_output_get_gamma_size(self.output) }
    }

    /// Sets the position of this output.
    pub fn set_position(&mut self, origin: Origin) {
        unsafe { wlr_output_set_position(self.output, origin.x, origin.y) }
    }

    /// Set the scale applied to this output.
    pub fn set_scale(&mut self, scale: c_float) {
        unsafe { wlr_output_set_scale(self.output, scale) }
    }

    pub fn damage(&mut self) -> &mut output::Damage {
        &mut *self.damage
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        // NOTE
        // We do _not_ need to call wlr_output_destroy for the output
        // That is handled by the backend automatically

        // NOTE
        // We do _not_ need to call wlr_output_damage_destroy for the output,
        // that is handled automatically by the listeners in wlroots.
        if Rc::strong_count(&self.liveliness) == 1 {
            wlr_log!(WLR_DEBUG, "Dropped output {:p}", self.output);
            let weak_count = Rc::weak_count(&self.liveliness);
            if weak_count > 0 {
                wlr_log!(WLR_DEBUG,
                         "Still {} weak pointers to Output {:p}",
                         weak_count,
                         self.output);
            }
        } else {
            return
        }
        // TODO Move back up in the some after NLL is a thing.
        unsafe {
            self.remove_from_output_layout();
            let _ = Box::from_raw((*self.output).data as *mut OutputState);
        }
    }
}

impl Handleable<*mut wlr_output_damage, wlr_output> for Output {
    #[doc(hidden)]
    unsafe fn from_ptr(ptr: *mut wlr_output) -> Self where Self: Sized {
        let data = Box::from_raw((*ptr).data as *mut OutputState);
        let handle = data.handle.clone();
        let damage = data.damage;
        (*ptr).data = Box::into_raw(data) as *mut _;
        Output { liveliness: handle.upgrade().unwrap(),
                 damage: ManuallyDrop::new(output::Damage::from_ptr(damage)),
                 output: ptr}

    }

    #[doc(hidden)]
    unsafe fn as_ptr(&self) -> *mut wlr_output {
        self.output
    }

    #[doc(hidden)]
    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> where Self: Sized {
        let liveliness = handle.handle
            .upgrade()
            .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(Output { liveliness,
                    damage: ManuallyDrop::new(output::Damage::from_ptr(handle.data)),
                    output: handle.as_ptr() })
    }

    fn weak_reference(&self) -> Handle {
        Handle { ptr: self.output,
                 handle: Rc::downgrade(&self.liveliness),
                 data: unsafe { self.damage.as_ptr() },
                 _marker: std::marker::PhantomData }
    }
}
