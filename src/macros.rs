/// Gets the offset of a field. Used by container_of!
#[macro_export]
macro_rules! offset_of(
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
);

/// Gets the parent struct from a pointer.
/// VERY unsafe. The parent struct _must_ be repr(C), and the
/// type passed to this macro _must_ match the type of the parent.
#[macro_export]
macro_rules! container_of (
    ($ptr: expr, $container: ty, $field: ident) => {
        ($ptr as *mut u8).offset(-(offset_of!($container, $field) as isize)) as *mut $container
    }
);

/// Convert a literal string to a C string.
/// Note: Does not check for internal nulls, nor does it do any conversions on
/// the grapheme clustors. Just passes the bytes as is.
/// So probably only works on ASCII.
#[macro_export]
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

/// Defines a new struct that contains a variable number of listeners that
/// will trigger unsafe user-defined callbacks.
///
/// The structure that is defined is repr(C), has one `data` field with the
/// given user type, and a field for each `$listener`.
///
/// Each `$listener` has a getter method that lets you get the pointer to the
/// listener. This method is unsafe, since it returns a raw pointer.
/// To use it correctly, you need to ensure that the data it refers to never
/// moves (e.g keep it in a box). The primary purpose of this method is to pass
/// the listener pointer to other methods to register it for a Wayland event.
/// **A listener can only be registered to one event at a time**.
///
/// Finally, it also takes in a body for each `$listener` that is called
/// every time the event that is later hooked up to it is fired.
/// This method is inherently unsafe, because the user data hasn't been cast
/// from the void pointer yet. It is the user's job to write this safely.
/// To highlight this fact, the body of the function must be prefixed with
/// `unsafe`.
///
/// # Example
/// ```
/// // Handles input addition and removal.
/// pub trait InputManagerHandler {
///     // Callback triggered when an input device is added.
///     fn input_added(&mut self, Device);
/// }
/// wayland_listener!(
///     // The name of the structure that will be defined.
///     InputManager,
///     // The type that's stored in the `data` field.
///     // Note that we use a Box here to achieve dynamic dispatch,
///     // it's not required for this type to be in a box.
///     Box<InputManagerHandler>,
///     [
///         // Adds a new listener called `add_listener`.
///         add_listener =>
///         // Adds an unsafe function called `add_notify` that is triggered
///         // whenever add_listener is activated from a Wayland event.
///         add_notify: |input_manager: &mut Box<InputManagerHandler>,
///                      data: *mut libc::c_void,| unsafe {
/// // Call the method defined above, wrapping it in a safe
/// interface.
/// // It is your job to ensure that the code in here doesn't
/// trigger UB!
/// input_manager.input_added(Device::from_ptr(data as *mut
/// wlr_input_device))
///         };
///     ]
/// );
/// ```
///
/// # Unsafety
/// Note that the purpose of this macro is to make it easy to generate unsafe
/// boiler plate for using listeners with Rust data.
///
/// However, there are a few things this macro doesn't protect against.
///
/// First and foremost, the data cannot move. The listeners assume that the
/// structure will never move, so in order to defend against this the generated
/// `new` method returns a Box version. **Do not move out of the box**.
///
/// Second, this macro doesn't protect against the stored data being unsized.
/// Passing a pointer of unsized data to C is UB, don't do it.
#[macro_export]
macro_rules! wayland_listener {
    ($struct_name: ident, $data: ty, $([
        $($listener: ident => $listener_func: ident :
          |$($func_arg:ident: $func_type:ty,)*| unsafe $body: block;)*])+) => {
        #[repr(C)]
        pub(crate) struct $struct_name {
            data: $data,
            $($($listener: $crate::wlroots_sys::wl_listener),*)*
        }

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

            $($(pub unsafe extern "C" fn $listener(&mut self)
                                                   -> *mut $crate::wlroots_sys::wl_listener {
                &mut self.$listener as *mut _
            })*)*

            $($(pub unsafe extern "C" fn $listener_func(listener:
                                                        *mut $crate::wlroots_sys::wl_listener,
                                                        data: *mut libc::c_void) {
                let manager: &mut $struct_name = &mut (*container_of!(listener,
                                                                      $struct_name,
                                                                      $listener));
                (|$($func_arg: $func_type,)*| { $body })(manager, data);
            })*)*
        }
    }
}
