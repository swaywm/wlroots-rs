use std::marker::PhantomData;
use std::path::Path;

use libc::{c_int, c_uint, c_char};
use wlroots_sys::{wl_display, wlr_session, wlr_session_create, wlr_session_destroy,
                  wlr_session_open_file, wlr_session_close_file, wlr_session_signal_add,
                  wlr_session_change_vt, wl_listener, wl_signal, udev, udev_monitor, wlr_device,
                  dev_t};

use utils::safe_as_cstring;

pub struct Device<'session> {
    device: *mut wlr_device,
    phantom: PhantomData<&'session ()>
}

pub struct Session<'session> {
    session: *mut wlr_session,
    phantom: PhantomData<&'session ()>
}

impl <'session> Device<'session> {
    unsafe fn from_ptr<'unbound>(device: *mut wlr_device) -> Device<'unbound> {
        Device { device, phantom: PhantomData }
    }

    pub fn fd(&self) -> c_int {
        unsafe { (*self.device).fd }
    }

    pub fn dev(&self) -> dev_t {
        unsafe { (*self.device).dev }
    }
}

impl <'session> Session<'session> {
    pub(crate) unsafe fn from_ptr(session: *mut wlr_session) -> Self {
        Session { session, phantom: PhantomData }
    }

	  /// Signal for when the session becomes active/inactive.
    /// It's called when we swap virtual terminal.
    pub fn session_signal(&self) -> wl_signal {
        unsafe { (*self.session).session_signal }
    }

    pub fn active(&self) -> bool {
        unsafe { (*self.session).active }
    }

    pub fn vtnr(&self) -> c_uint {
        unsafe { (*self.session).vtnr }
    }

    pub fn seat(&self) -> [c_char; 8] {
        unsafe { (*self.session).seat }
    }

    pub fn udev(&self) -> *mut udev {
        unsafe { (*self.session).udev }
    }

    pub fn udev_monitor(&self) -> *mut udev_monitor {
        unsafe { (*self.session).mon }
    }

    pub fn devices(&self) -> Vec<Device<'session>> {
        unsafe {
            let mut devices = Vec::new();
            wl_list_for_each!((*self.session).devices,
                              link,
                              (device: wlr_device) => {
                                  devices.push(Device::from_ptr(device))
                              });
            devices
        }
    }

    /// Changes the virtual terminal.
    pub fn change_vt(&mut self, vt: c_uint) -> bool {
        unsafe {
            wlr_session_change_vt(self.session, vt)
        }
    }

    pub unsafe fn as_ptr(&self) -> *mut wlr_session {
        self.session
    }

    /// Opens a session, taking control of the current virtual terminal.
    /// This should not be called if another program is already in control
    /// of the terminal (Xorg, another Wayland compositor, etc.).
    ///
    /// If logind support is not enabled, you must have CAP_SYS_ADMIN or be root.
    /// It is safe to drop privileges after this is called.
    ///
    /// Returns `None` on error.
    pub unsafe fn new(display: *mut wl_display) -> Option<Self> {
        let session = wlr_session_create(display);
        if session.is_null() {
            None
        } else {
            Some(Session {
                session,
                phantom: PhantomData
            })
        }
    }

    /// Closes a previously opened session and restores the virtual terminal.
    /// You should call Session::close_file on each files you opened
    /// with Session::open_file before you call this.
    pub unsafe fn destroy(self) {
        wlr_session_destroy(self.session)
    }

    /// Opens the file at path.
    /// This can only be used to open DRM or evdev (input) devices.
    ///
    /// When the session becomes inactive:
    /// - DRM files lose their DRM master status
    /// - evdev files become invalid and should be closed
    ///
    /// Returns -errno on error.
    pub unsafe fn open_file<P: AsRef<Path>>(&mut self, path: P) -> c_int {
        let path_c = safe_as_cstring(path.as_ref().to_str().expect("Path was not UTF-8"));
        wlr_session_open_file(self.session, path_c.as_ptr())
    }

    pub unsafe fn close_file<P: AsRef<Path>>(&mut self, fd: c_int) {
        wlr_session_close_file(self.session, fd);
    }

    pub unsafe fn signal_add(&mut self, fd: c_int, listener: *mut wl_listener) {
        wlr_session_signal_add(self.session, fd, listener)
    }
}
