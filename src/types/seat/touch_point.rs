use wlroots_sys::wlr_touch_point;

#[derive(Clone)]
pub struct TouchPoint {
    touch_point: *mut wlr_touch_point
}

/// Wrapper around a touch id. It is valid, as it is only returned from wlroots.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TouchId(i32);

// Note that we implement `Into` here because we _don't_ want to be able to
// convert from any i32 into a `TouchId`
impl Into<i32> for TouchId {
    fn into(self) -> i32 {
        self.0
    }
}

impl TouchPoint {
    /// Get the touch id associated for this point.
    pub fn touch_id(&self) -> TouchId {
        unsafe { TouchId((*self.touch_point).touch_id) }
    }

    #[allow(dead_code)]
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_touch_point {
        self.touch_point
    }

    pub(crate) unsafe fn from_ptr(touch_point: *mut wlr_touch_point) -> Self {
        TouchPoint { touch_point }
    }
}
