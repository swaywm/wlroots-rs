use wlroots_sys::wlr_session;

pub struct Session {
    session: *mut wlr_session
}

impl Session {
    pub(crate) unsafe fn from_ptr(session: *mut wlr_session) -> Self {
        Session { session }
    }

    pub unsafe fn as_ptr(&self) -> *mut wlr_session {
        self.session
    }
}
