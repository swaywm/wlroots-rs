//! TODO Documentation

use libc::c_int;
use std::marker::PhantomData;

use wlroots_sys::wlr_surface_state;

use Surface;

/// Surface state as reported by wlroots.
#[derive(Debug)]
pub struct SurfaceState<'surface> {
    state: *mut wlr_surface_state,
    phantom: PhantomData<&'surface Surface>
}

impl<'surface> SurfaceState<'surface> {
    /// Create a new subsurface from the given surface.
    ///
    /// # Safety
    /// Since we rely on the surface providing a valid surface state,
    /// this function is marked unsafe.
    ///
    /// However, the lifetimes should pose no problems.
    pub(crate) unsafe fn new(_surface: &'surface mut Surface,
                             state: *mut wlr_surface_state)
                             -> SurfaceState<'surface> {
        SurfaceState { state,
                       phantom: PhantomData }
    }

    /// Get the size of the sub surface.
    ///
    /// Return value is in (width, height) format.
    pub fn size(&self) -> (c_int, c_int) {
        unsafe { ((*self.state).width, (*self.state).height) }
    }
}
