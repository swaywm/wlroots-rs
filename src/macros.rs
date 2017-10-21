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

/// Define a struct with some listeners that can call user-defined callbacks
/// every time some Wayland event fires.
macro_rules! define_listener {
    // FIXME TODO Impl drop for the listener data
    ($struct_name: ident, $data: ty, $([$($listener: ident, $listener_func: ident : |$($func_arg:ident: $func_type:ty,)*| unsafe $body: block;)*])+) => {
        #[repr(C)]
        pub struct $struct_name {
            data: $data,
            $($($listener: $crate::wlroots_sys::wl_listener),*)*
        }

        // TODO Allow a pattern that does everything here, but it makes a method
        // that just takes in data and inits the functions to ones defined by the user of the macro.

        impl $struct_name {
            pub fn new(data: $data) -> Box<$struct_name> {
                use $crate::wayland_sys::server::WAYLAND_SERVER_HANDLE;
                Box::new($struct_name {
                    data,
                    $($($listener: unsafe {
                        // NOTE Rationale for zeroed memory:
                        // * Need to pass a pointer to wl_list_init
                        // * The list is initialized by Wayland, which doesn't "drop"
                        // * The listener is written to without dropping any of the data
                        let mut listener: $crate::wlroots_sys::wl_listener = ::std::mem::zeroed();
                        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                      wl_list_init,
                                      &mut listener.link as *mut _ as _);
                        ::std::ptr::write(&mut listener.notify, Some($struct_name::$listener_func));
                        listener
                    }),*)*
                })
            }

            $($(pub unsafe extern "C" fn $listener(&mut self) -> *mut $crate::wlroots_sys::wl_listener {
                &mut self.$listener as *mut _
            })*)*

            $($(pub unsafe extern "C" fn $listener_func(listener: *mut $crate::wlroots_sys::wl_listener, data: *mut libc::c_void) {
                let manager_wrapper: *mut $struct_name = container_of!(listener,
                                                    $struct_name,
                                                    $listener);
                let manager: &mut $data = &mut (*manager_wrapper).data;
                (|$($func_arg: $func_type,)*| { $body })(manager, data);
            })*)*
        }
    }
}
