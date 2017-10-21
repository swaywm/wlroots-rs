/// Gets the offset of a field. Used by container_of!
macro_rules! offset_of(
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
);

/// Gets the parent struct from a pointer.
/// VERY unsafe. The parent struct _must_ be repr(C), and the
/// type passed to this macro _must_ match the type of the parent.
macro_rules! container_of (
    ($ptr: expr, $container: ty, $field: ident) => {
        ($ptr as *mut u8).offset(-(offset_of!($container, $field) as isize)) as *mut $container
    }
);

/// Convert a literal string to a C string.
/// Note: Does not check for internal nulls, nor does it do any conversions on
/// the grapheme clustors. Just passes the bytes as is.
/// So probably only works on ASCII.
macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    }
}

#[macro_export]
/// Logs a message using wlroots' logging capability.
macro_rules! wlr_log {
    ($verb: expr, $($msg:tt)*) => {{
        //format!($($msg)*)
        use $crate::wlroots_sys::_wlr_log;
        use $crate::wlroots_sys::log_importance_t::*;
        use ::std::ffi::CString;
        unsafe {
            let fmt = CString::new(format!($($msg)*))
                .expect("Could not convert log message to C string");
            let raw = fmt.into_raw();
            _wlr_log($verb, c_str!("[%s:%lu] %s"),
                    c_str!(file!()), line!(), raw);
            // Deallocate string
            CString::from_raw(raw);
        }
    }}
}
