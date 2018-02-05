//! TODO Documentation

use std::{panic, ptr};
use std::marker::PhantomData;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use libc;
use wlroots_sys::{timespec, wlr_surface, wlr_surface_get_main_surface, wlr_surface_get_matrix,
                  wlr_surface_has_buffer, wlr_surface_make_subsurface, wlr_surface_send_enter,
                  wlr_surface_send_frame_done, wlr_surface_send_leave, wlr_surface_state};

use Output;
use errors::{UpgradeHandleErr, UpgradeHandleResult};
use render::Texture;
use utils::c_to_rust_string;

/// The state stored in the wlr_surface user data.
struct InternalSurfaceState {
    /// Used to reconstruct a SurfaceHandle from just an *mut wlr_surface.
    handle: Weak<AtomicBool>
}

// TODO Move
pub struct Subsurface;
pub struct SubsurfaceHandle;

/// Surface state as reported by wlroots.
pub struct SurfaceState<'surface> {
    state: *mut wlr_surface_state,
    phantom: PhantomData<&'surface Surface>
}

/// A Wayland object that represents the data that we display on the screen.
///
/// Most surfaces come from Wayland clients, though they can also be created
/// by the compositor directly.
#[derive(Debug)]
pub struct Surface {
    /// The structe that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `SurfaceHandle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `SurfaceHandle`.
    liveliness: Option<Rc<AtomicBool>>,
    /// The pointer to the wlroots object that wraps a wl_surface.
    surface: *mut wlr_surface
}

/// See `Surface` for more information on how to use this structure.
#[derive(Clone, Debug)]
pub struct SurfaceHandle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the pointer associated with this handle,
    /// this can no longer be used.
    handle: Weak<AtomicBool>,
    /// The pointer to the wlroots object that wraps a wl_surface.
    surface: *mut wlr_surface
}

impl Surface {
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_surface {
        self.surface
    }

    pub(crate) unsafe fn from_ptr(surface: *mut wlr_surface) -> Self {
        if !(*surface).data.is_null() {
            panic!("Tried to construct a Surface from an already initialized wlr_surface");
        }
        let liveliness = Rc::new(AtomicBool::new(false));
        let handle = Rc::downgrade(&liveliness);
        (*surface).data = Box::into_raw(Box::new(InternalSurfaceState { handle })) as _;
        let liveliness = Some(liveliness);
        Surface { liveliness,
                  surface }
    }

    /// Get the surface state.
    pub fn current_state<'surface>(&'surface mut self) -> SurfaceState<'surface> {
        unsafe {
            SurfaceState { state: (*self.surface).current,
                           phantom: PhantomData }
        }
    }

    /// Get the pending surface state.
    pub fn pending_state<'surface>(&'surface mut self) -> SurfaceState<'surface> {
        unsafe {
            SurfaceState { state: (*self.surface).pending,
                           phantom: PhantomData }
        }
    }

    /// Get the subsurface.
    pub fn subsurface(&self) -> Subsurface {
        Subsurface
    }

    /// Get the texture of this surface.
    pub fn texture(&self) -> Texture {
        unsafe { Texture::from_ptr((*self.surface).texture) }
    }

    /// Get the lifetime bound role (if one exists) for this surface.
    pub fn role(&self) -> Option<String> {
        unsafe { c_to_rust_string((*self.surface).role) }
    }

    /// Gets a matrix you can pass into wlr_render_with_matrix to display this
    /// surface.
    ///
    /// `matrix` is the output matrix, `projection` is the wlr_output
    /// projection matrix, and `transform` is any additional transformations you want
    /// to perform on the surface (or None/the identity matrix if you don't).
    ///
    /// `transform` is used before the surface is scaled, so its geometry extends
    /// from 0 to 1 in both dimensions.
    pub fn get_matrix<'a, T>(&mut self,
                             matrix: &mut [f32; 16],
                             projection: &[f32; 16],
                             transform: T)
        where T: Into<Option<&'a [f32; 16]>>
    {
        let transform = transform.into()
                                 .map(|transform| transform as *const _)
                                 .unwrap_or_else(|| ptr::null());
        unsafe { wlr_surface_get_matrix(self.surface, matrix, projection, transform) }
    }

    /// Whether or not this surface currently has an attached buffer.
    ///
    /// A surface has an attached buffer when it commits with a non-null buffer in its pending
    /// state.
    ///
    /// A surface will not have a buffer if it has never committed one, has
    /// committed a null buffer, or something went wrong with uploading the buffer.
    pub fn has_buffer(&self) -> bool {
        unsafe { wlr_surface_has_buffer(self.surface) }
    }

    /// Find a subsurface within this surface at the surface-local coordinates.
    ///
    /// Returns the surface and coordinates in the topmost surface coordinate system
    /// or None if no subsurface is found at that location.
    pub fn subsurface_at(&mut self,
                         sx: f32,
                         sy: f32,
                         sub_x: &mut f32,
                         sub_y: &mut f32)
                         -> Option<SubsurfaceHandle> {
        None
    }

    /// Create the subsurface implementation for this surface.
    pub fn make_subsurface(&mut self, parent: &mut Surface, id: u32) {
        unsafe { wlr_surface_make_subsurface(self.surface, parent.as_ptr(), id) }
    }

    /// Get the top of the subsurface tree for this surface.
    pub fn get_main_surface(&self) -> Option<SurfaceHandle> {
        unsafe {
            let surface = wlr_surface_get_main_surface(self.surface);
            if surface.is_null() {
                None
            } else {
                Some(SurfaceHandle::from_ptr(surface))
            }
        }
    }

    pub fn send_enter(&mut self, output: &mut Output) {
        unsafe { wlr_surface_send_enter(self.surface, output.as_ptr()) }
    }

    pub fn send_leave(&mut self, output: &mut Output) {
        unsafe { wlr_surface_send_leave(self.surface, output.as_ptr()) }
    }

    /// Send the frame done event.
    pub fn send_frame_done(&mut self, duration: Duration) {
        unsafe {
            // FIXME
            // This is converting from a u64 -> i64
            // Something bad could happen!
            let when = timespec { tv_sec: duration.as_secs() as i64,
                                  tv_nsec: duration.subsec_nanos() as i64 };
            wlr_surface_send_frame_done(self.surface, &when);
        }
    }

    /// Get the matrix used to convert the internal byte buffer to use in the
    /// surface.
    pub fn buffer_to_surface_matrix(&self) -> [f32; 16] {
        unsafe { (*self.surface).buffer_to_surface_matrix }
    }

    /// Get the matrix used to convert the surface back to the internal byte
    /// buffer.
    pub fn surface_to_buffer_matrix(&self) -> [f32; 16] {
        unsafe { (*self.surface).surface_to_buffer_matrix }
    }

    /// Manually set the lock used to determine if a double-borrow is
    /// occuring on this structure.
    ///
    /// # Panics
    /// Panics when trying to set the lock on an upgraded handle.
    pub(crate) unsafe fn set_lock(&self, val: bool) {
        self.liveliness.as_ref()
            .expect("Tried to set lock on borrowed Surface")
            .store(val, Ordering::Release);
    }

    unsafe fn from_handle(handle: &SurfaceHandle) -> Self {
        Surface { liveliness: None,
                  surface: handle.surface }
    }
}

impl SurfaceHandle {
    /// Creates an SurfaceHandle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    pub(crate) unsafe fn from_ptr(surface: *mut wlr_surface) -> Self {
        let data = (*surface).data as *mut InternalSurfaceState;
        let handle = (*data).handle.clone();
        SurfaceHandle { handle, surface }
    }

    /// Upgrades the surface handle to a reference to the backing `Surface`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Surface`
    /// which may live forever..
    /// But no surface lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> UpgradeHandleResult<Surface> {
        self.handle.upgrade()
            .ok_or(UpgradeHandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                if check.load(Ordering::Acquire) {
                    return Err(UpgradeHandleErr::AlreadyBorrowed)
                }
                check.store(true, Ordering::Release);
                Ok(Surface::from_handle(self))
            })
    }

    /// Run a function on the referenced Surface, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the surface
    /// to a short lived scope of an anonymous function,
    /// this function ensures the Surface does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Surface`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&mut self, runner: F) -> UpgradeHandleResult<R>
        where F: FnOnce(&mut Surface) -> R
    {
        let mut surface = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut surface)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.load(Ordering::Acquire) {
                                          wlr_log!(L_ERROR,
                                                   "After running surface callback, mutable lock \
                                                    was false for: {:?}",
                                                   surface);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.store(false, Ordering::Release);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }
}

impl<'surface> SurfaceState<'surface> {
    pub fn width(&self) -> libc::c_int {
        unsafe { (*self.state).width }
    }

    pub fn height(&self) -> libc::c_int {
        unsafe { (*self.state).height }
    }
}
