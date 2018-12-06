//! TODO Documentation

use std::{cell::Cell, rc::{Rc, Weak}, panic, ptr};

use libc::c_void;
use wlroots_sys::{wlr_xdg_popup, wlr_xdg_surface, wlr_xdg_surface_ping,
                  wlr_xdg_surface_role, wlr_xdg_surface_send_close,
                  wlr_xdg_surface_surface_at, wlr_xdg_toplevel,
                  wlr_xdg_toplevel_set_activated, wlr_xdg_toplevel_set_fullscreen,
                  wlr_xdg_toplevel_set_maximized, wlr_xdg_toplevel_set_resizing,
                  wlr_xdg_toplevel_set_size, wlr_xdg_toplevel_state,
                  wlr_xdg_surface_for_each_surface, wlr_surface, wlr_xdg_surface_from_wlr_surface};


use {area::Area,
     errors::{HandleErr, HandleResult},
     seat,
     surface,
     utils::c_to_rust_string};
pub use manager::{xdg_shell_manager::*, xdg_shell_handler::*};
pub use events::xdg_shell_events as event;

/// Used internally to reclaim a handle from just a *mut wlr_xdg_surface.
pub(crate) struct SurfaceState {
    /// Pointer to the backing storage.
    pub(crate) shell: *mut XdgShell,
    handle: Weak<Cell<bool>>,
    shell_state: Option<ShellState>
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct TopLevel {
    shell_surface: *mut wlr_xdg_surface,
    toplevel: *mut wlr_xdg_toplevel
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Popup {
    shell_surface: *mut wlr_xdg_surface,
    popup: *mut wlr_xdg_popup
}

/// A tagged enum of the different roles used by the xdg shell.
///
/// Uses the tag to disambiguate the union in `wlr_xdg_surface`.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ShellState {
    TopLevel(TopLevel),
    Popup(Popup)
}

#[derive(Debug)]
pub struct Surface {
    liveliness: Rc<Cell<bool>>,
    state: Option<ShellState>,
    shell_surface: *mut wlr_xdg_surface
}

#[derive(Debug)]
pub struct Handle {
    state: Option<ShellState>,
    handle: Weak<Cell<bool>>,
    shell_surface: *mut wlr_xdg_surface
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        let state = match self.state {
            None => None,
            Some(ref state) => Some(unsafe { state.clone() })
        };
        Handle { state,
                                  handle: self.handle.clone(),
                                  shell_surface: self.shell_surface }
    }
}

impl Surface {
    pub(crate) unsafe fn new<T>(shell_surface: *mut wlr_xdg_surface, state: T) -> Self
        where T: Into<Option<ShellState>>
    {
        let state = state.into();
        (*shell_surface).data = ptr::null_mut();
        let liveliness = Rc::new(Cell::new(false));
        let shell_state =
            Box::new(SurfaceState { shell: ptr::null_mut(),
                                            handle: Rc::downgrade(&liveliness),
                                            shell_state: match state {
                                                None => None,
                                                Some(ref state) => Some(state.clone())
                                            } });
        (*shell_surface).data = Box::into_raw(shell_state) as *mut _;
        Surface { liveliness,
                          state: state,
                          shell_surface }
    }

    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let liveliness = handle.handle
                               .upgrade()
                               .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(Surface { liveliness,
                               state: handle.clone().state,
                               shell_surface: handle.as_ptr() })
    }

    /// Gets the surface used by this XDG shell.
    pub fn surface(&mut self) -> surface::Handle {
        unsafe {
            let surface = (*self.shell_surface).surface;
            if surface.is_null() {
                panic!("xdg shell had a null surface!")
            }
            surface::Handle::from_ptr(surface)
        }
    }

    /// Get the role of this XDG surface.
    pub fn role(&self) -> wlr_xdg_surface_role {
        unsafe { (*self.shell_surface).role }
    }

    pub fn state(&mut self) -> Option<&mut ShellState> {
        self.state.as_mut()
    }

    /// Determines if this XDG shell surface has been configured or not.
    pub fn configured(&self) -> bool {
        unsafe { (*self.shell_surface).configured }
    }

    pub fn added(&self) -> bool {
        unsafe { (*self.shell_surface).added }
    }

    pub fn configure_serial(&self) -> u32 {
        unsafe { (*self.shell_surface).configure_serial }
    }

    pub fn configure_next_serial(&self) -> u32 {
        unsafe { (*self.shell_surface).configure_next_serial }
    }

    pub fn has_next_geometry(&self) -> bool {
        unsafe { (*self.shell_surface).has_next_geometry }
    }

    pub fn next_geometry(&self) -> Area {
        unsafe { Area::from_box((*self.shell_surface).next_geometry) }
    }

    pub fn geometry(&self) -> Area {
        unsafe { Area::from_box((*self.shell_surface).geometry) }
    }

    /// Send a ping to the surface.
    ///
    /// If the surface does not respond with a pong within a reasonable amount of time,
    /// the ping timeout event will be emitted.
    pub fn ping(&mut self) {
        unsafe {
            wlr_xdg_surface_ping(self.shell_surface);
        }
    }

    /// Find a surface within this surface at the surface-local coordinates.
    ///
    /// Returns the popup and coordinates in the topmost surface coordinate system
    /// or None if no popup is found at that location.
    pub fn surface_at(&mut self,
                      sx: f64,
                      sy: f64,
                      sub_sx: &mut f64,
                      sub_sy: &mut f64)
                      -> Option<surface::Handle> {
        unsafe {
            let sub_surface =
                wlr_xdg_surface_surface_at(self.shell_surface, sx, sy, sub_sx, sub_sy);
            if sub_surface.is_null() {
                None
            } else {
                Some(surface::Handle::from_ptr(sub_surface))
            }
        }
    }

    pub fn for_each_surface<F>(&self, mut iterator: F)
            where F: FnMut(surface::Handle, i32, i32) {
        let mut iterator_ref: &mut FnMut(surface::Handle, i32, i32) = &mut iterator;
        unsafe {
            unsafe extern "C" fn c_iterator(wlr_surface: *mut wlr_surface, sx: i32, sy: i32, data: *mut c_void) {
                let iterator_fn = &mut *(data as *mut &mut FnMut(surface::Handle, i32, i32));
                let surface = surface::Handle::from_ptr(wlr_surface);
                iterator_fn(surface, sx, sy);
            }
            let iterator_ptr: *mut c_void = &mut iterator_ref as *mut _ as *mut c_void;
            wlr_xdg_surface_for_each_surface(self.shell_surface, Some(c_iterator), iterator_ptr);
        }
    }

    /// Creates a weak reference to an `Surface`.
    ///
    /// # Panics
    /// If this `Surface` is a previously upgraded `xdg_shell::Handle`,
    /// then this function will panic.
    pub fn weak_reference(&self) -> Handle {
        Handle { handle: Rc::downgrade(&self.liveliness),
                                  state: match self.state {
                                      None => None,
                                      Some(ref state) => unsafe { Some(state.clone()) }
                                  },
                                  shell_surface: self.shell_surface }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) == 1 {
            wlr_log!(WLR_DEBUG, "Dropped xdg shell {:p}", self.shell_surface);
            let weak_count = Rc::weak_count(&self.liveliness);
            if weak_count > 0 {
                wlr_log!(WLR_DEBUG,
                         "Still {} weak pointers to xdg shell {:p}",
                         weak_count,
                         self.shell_surface);
            }
        } else {
            return
        }
        unsafe {
            let _ = Box::from_raw((*self.shell_surface).data as *mut SurfaceState);
        }
    }
}

impl Handle {
    /// Constructs a new xdg_shell::Handle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            Handle { handle: Weak::new(),
                                      state: None,
                                      shell_surface: ptr::null_mut() }
        }
    }

    /// If the surface is an XDG surface, get a handle to the XDG surface.
    pub fn from_surface(surface: &surface::Surface) -> Option<Handle> {
        unsafe {
            if !surface.is_xdg_surface() {
                None
            } else {
                let xdg_surface_ptr = wlr_xdg_surface_from_wlr_surface(surface.as_ptr());
                Some(Handle::from_ptr(xdg_surface_ptr))
            }
        }
    }

    /// Creates a xdg_shell::Handle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    pub(crate) unsafe fn from_ptr(shell_surface: *mut wlr_xdg_surface) -> Self {
        let data = (*shell_surface).data as *mut SurfaceState;
        if data.is_null() {
            panic!("Cannot construct handle from a shell surface that has not been set up!");
        }
        let handle = (*data).handle.clone();
        let state = match (*data).shell_state {
            None => None,
            Some(ref state) => Some(unsafe { state.clone() })
        };
        Handle { handle,
                                  state,
                                  shell_surface }
    }

    /// Upgrades the wayland shell handle to a reference to the backing `Surface`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Surface`
    /// which may live forever..
    /// But no surface lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<Surface> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let shell_surface = Surface::from_handle(self)?;
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.set(true);
                Ok(shell_surface)
            })
    }

    /// Run a function on the referenced Surface, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the output
    /// to a short lived scope of an anonymous function,
    /// this function ensures the Surface does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Output`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut Surface) -> R
    {
        let mut xdg_surface = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut xdg_surface)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(WLR_ERROR,
                                                   "After running Surface callback, \
                                                    mutable lock was false for: {:?}",
                                                   xdg_surface);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.set(false);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    unsafe fn as_ptr(&self) -> *mut wlr_xdg_surface {
        self.shell_surface
    }
}

impl Default for Handle {
    fn default() -> Self {
        Handle::new()
    }
}

impl PartialEq for Handle {
    fn eq(&self, other: &Handle) -> bool {
        self.shell_surface == other.shell_surface
    }
}

impl Eq for Handle {}

impl TopLevel {
    pub(crate) unsafe fn from_shell(shell_surface: *mut wlr_xdg_surface,
                                    toplevel: *mut wlr_xdg_toplevel)
                                    -> TopLevel {
        TopLevel { shell_surface,
                        toplevel }
    }

    /// Get the title associated with this XDG shell toplevel.
    pub fn title(&self) -> String {
        unsafe { c_to_rust_string((*self.toplevel).title).expect("Could not parse class as UTF-8") }
    }

    /// Get the app id associated with this XDG shell toplevel.
    pub fn app_id(&self) -> String {
        unsafe {
            c_to_rust_string((*self.toplevel).app_id).expect("Could not parse class as UTF-8")
        }
    }

    /// Get a handle to the base surface of the xdg tree.
    pub fn base(&self) -> Handle {
        unsafe { Handle::from_ptr((*self.toplevel).base) }
    }

    /// Get a handle to the parent surface of the xdg tree.
    pub fn parent(&self) -> Handle {
        unsafe { Handle::from_ptr((*self.toplevel).parent) }
    }

    pub fn added(&self) -> bool {
        unsafe { (*self.toplevel).added }
    }

    /// Get the pending client state.
    pub fn client_pending_state(&self) -> wlr_xdg_toplevel_state {
        unsafe { (*self.toplevel).client_pending }
    }

    /// Get the pending server state.
    pub fn server_pending_state(&self) -> wlr_xdg_toplevel_state {
        unsafe { (*self.toplevel).server_pending }
    }

    /// Get the current configure state.
    pub fn current_state(&self) -> wlr_xdg_toplevel_state {
        unsafe { (*self.toplevel).current }
    }

    /// Request that this toplevel surface be the given size.
    ///
    /// Returns the associated configure serial.
    pub fn set_size(&mut self, width: u32, height: u32) -> u32 {
        unsafe { wlr_xdg_toplevel_set_size(self.shell_surface, width, height) }
    }

    /// Request that this toplevel surface show itself in an activated or deactivated
    /// state.
    ///
    /// Returns the associated configure serial.
    pub fn set_activated(&mut self, activated: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_set_activated(self.shell_surface, activated) }
    }

    /// Request that this toplevel surface consider itself maximized or not
    /// maximized.
    ///
    /// Returns the associated configure serial.
    pub fn set_maximized(&mut self, maximized: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_set_maximized(self.shell_surface, maximized) }
    }

    /// Request that this toplevel surface consider itself fullscreen or not
    /// fullscreen.
    ///
    /// Returns the associated configure serial.
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_set_fullscreen(self.shell_surface, fullscreen) }
    }

    /// Request that this toplevel surface consider itself to be resizing or not
    /// resizing.
    ///
    /// Returns the associated configure serial.
    pub fn set_resizing(&mut self, resizing: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_set_resizing(self.shell_surface, resizing) }
    }

    /// Request that this toplevel surface closes.
    pub fn close(&mut self) {
        unsafe { wlr_xdg_surface_send_close(self.shell_surface) }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_xdg_toplevel {
        self.toplevel
    }
}

impl Popup {
    pub(crate) unsafe fn from_shell(shell_surface: *mut wlr_xdg_surface,
                                    popup: *mut wlr_xdg_popup)
                                    -> Popup {
        Popup { shell_surface,
                   popup }
    }

    /// Get a handle to the base surface of the xdg tree.
    pub fn base(&self) -> Handle {
        unsafe { Handle::from_ptr((*self.popup).base) }
    }

    /// Get a handle to the parent surface of the xdg tree.
    pub fn parent(&self) -> surface::Handle {
        unsafe { surface::Handle::from_ptr((*self.popup).parent) }
    }

    pub fn committed(&self) -> bool {
        unsafe { (*self.popup).committed }
    }

    /// Get a handle to the seat associated with this popup.
    pub fn seat_handle(&self) -> Option<seat::Handle> {
        unsafe {
            let seat = (*self.popup).seat;
            if seat.is_null() {
                None
            } else {
                Some(seat::Handle::from_ptr(seat))
            }
        }
    }

    pub fn geometry(&self) -> Area {
        unsafe { Area::from_box((*self.popup).geometry) }
    }
}

impl ShellState {
    /// Unsafe copy of the pointer
    unsafe fn clone(&self) -> Self {
        match *self {
            ShellState::TopLevel(TopLevel { shell_surface,
                                     toplevel }) => {
                ShellState::TopLevel(TopLevel { shell_surface,
                                         toplevel })
            }
            ShellState::Popup(Popup { shell_surface,
                               popup }) => {
                ShellState::Popup(Popup { shell_surface,
                                   popup })
            }
        }
    }
}
