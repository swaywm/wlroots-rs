//! TODO Documentation

use libc::c_int;
use std::marker::PhantomData;

use wlroots_sys::{wl_output_transform, wl_resource, wlr_surface_state, pixman_region32_t};

use Surface;

#[derive(Debug)]
#[repr(u32)]
/// Represents a change in the pending state.
///
/// When a particular bit is set, it means the field corresponding to it
/// will be updated for the current state on the next commit.
///
/// # Pending vs Current state
/// When this is set on the pending state, it means this field will be updated on
/// the next commit.
///
/// When it is set on the current state, it indicates what fields have changed
/// since the last commit.
pub enum InvalidState {
    Buffer = 1,
    SurfaceDamage = 2,
    BufferDamage = 4,
    OpaqueRegion = 8,
    InputRegion = 16,
    Transform = 32,
    Scale = 64,
    SubsurfacePosition = 128,
    FrameCallbackList = 256
}

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

    /// Gets the state of the sub surface.
    ///
    /// # Panics
    /// If the invalid state is in an undefined state, this will panic.
    pub fn invalid(&self) -> InvalidState {
        use InvalidState::*;
        unsafe {
            match (*self.state).invalid {
                1 => Buffer,
                2 => SurfaceDamage,
                4 => BufferDamage,
                8 => OpaqueRegion,
                16 => InputRegion,
                32 => Transform,
                64 => Scale,
                128 => SubsurfacePosition,
                256 => FrameCallbackList,
                invalid => {
                    wlr_log!(L_ERROR, "Invalid invalid state {}", invalid);
                    panic!("Invalid invalid state in wlr_surface_state")
                }
            }
        }
    }

    /// Get the position of the surface.
    ///
    /// Return value is in (sx, sy) format.
    pub fn position(&self) -> (i32, i32) {
        unsafe { ((*self.state).sx, (*self.state).sy) }
    }

    /// Get the position of the subsurface.
    ///
    /// Retrun value is in (x, y) format.
    pub fn subsurface_position(&self) -> (i32, i32) {
        unsafe { ((*self.state).subsurface_position.x, (*self.state).subsurface_position.y) }
    }

    /// Get the size of the sub surface.
    ///
    /// Return value is in (width, height) format.
    pub fn size(&self) -> (c_int, c_int) {
        unsafe { ((*self.state).width, (*self.state).height) }
    }

    /// Get the size of the buffer.
    ///
    /// Return value is iw (width, height) format.
    pub fn buffer_size(&self) -> (c_int, c_int) {
        unsafe { ((*self.state).buffer_width, (*self.state).buffer_height) }
    }

    /// Get the scale applied to the surface.
    pub fn scale(&self) -> i32 {
        unsafe { (*self.state).scale }
    }

    /// Get the output transform applied to the surface.
    pub fn transform(&self) -> wl_output_transform {
        unsafe { (*self.state).transform }
    }

    /// Gets the buffer of the surface.
    pub unsafe fn buffer(&self) -> *mut wl_resource {
        (*self.state).buffer
    }

    pub unsafe fn surface_damage(&self) -> pixman_region32_t {
        (*self.state).surface_damage
    }

    pub unsafe fn buffer_damage(&self) -> pixman_region32_t {
        (*self.state).buffer_damage
    }

    pub unsafe fn opaque(&self) -> pixman_region32_t {
        (*self.state).opaque
    }

    pub unsafe fn input(&self) -> pixman_region32_t {
        (*self.state).input
    }
}
