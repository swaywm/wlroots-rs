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

/// Iterates over a wl_list.
///
/// # Safety
/// It is not safe to delete an element while iterating over the list,
/// so don't do it!
macro_rules! wl_list_for_each {
    ($ptr: expr, $field: ident, ($pos: ident : $container: ty) => $body: block) => {
        let mut $pos: *mut $container;
        $pos = container_of!($ptr.next, $container, $field);
        loop {
            if &(*$pos).$field as *const _ == &$ptr as *const _ {
                break;
            }
            {
                $body
            }
            $pos = container_of!((*$pos).$field.next, $container, $field);
        }
    };
}

/// Logs a message using wlroots' logging capability.
///
/// Example:
/// ```rust,no_run,ignore
/// #[macro_use]
/// use wlroots::log::{init_logging, L_DEBUG, L_ERROR};
///
/// // Call this once, at the beginning of your program.
/// init_logging(WLR_DEBUG, None);
///
/// wlr_log!(L_DEBUG, "Hello world");
/// wlr_log!(L_ERROR, "Could not {:#?} the {}", foo, bar);
/// ```
#[macro_export]
macro_rules! wlr_log {
    ($verb: expr, $($msg:tt)*) => {{
        /// Convert a literal string to a C string.
        /// Note: Does not check for internal nulls, nor does it do any conversions on
        /// the grapheme clustors. Just passes the bytes as is.
        /// So probably only works on ASCII.
        macro_rules! c_str {
            ($s:expr) => {
                concat!($s, "\0").as_ptr()
                    as *const $crate::wlroots_sys::libc::c_char
            }
        }
        use $crate::wlroots_sys::_wlr_log;
        use $crate::wlroots_sys::wlr_log_importance::*;
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
/// ```rust,no_run,ignore
/// #[macro_use] extern crate wlroots;
/// extern crate wlroots_sys;
/// #[macro_use] extern crate wayland_sys;
/// extern crate libc;
///
/// use wlroots::InputDevice;
/// use wlroots_sys::wlr_input_device;
///
/// // Handles input addition and removal.
/// pub trait InputManagerHandler {
///     // Callback triggered when an input device is added.
///     fn input_added(&mut self, InputDevice);
/// }
///
/// wayland_listener!(
///     // The name of the structure that will be defined.
///     pub(crate) InputManager,
///     // The type that's stored in the `data` field.
///     // Note that we use a Box here to achieve dynamic dispatch,
///     // it's not required for this type to be in a box.
///     Box<InputManagerHandler>,
///     [
///         // Adds a new listener called `add_listener`.
///         // Adds an unsafe function called `add_notify` that is triggered
///         // whenever add_listener is activated from a Wayland event.
///         add_listener => add_notify: |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
///             let ref mut manager = this.data;
///             // Call the method defined above, wrapping it in a safe interface.
///             // It is your job to ensure that the code in here doesn't trigger UB!
///             manager.input_added(InputDevice::from_ptr(data as *mut wlr_input_device))
///         };
///     ]
/// );
/// # fn main() {}
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
macro_rules! wayland_listener {
    ($pub: vis $struct_name: ident, $data: ty, $([
        $($listener: ident => $listener_func: ident :
          |$($func_arg:ident: $func_type:ty,)*| unsafe $body: block;)*])+) => {
        #[repr(C)]
        $pub struct $struct_name {
            data: $data,
            $($($listener: $crate::wlroots_sys::wl_listener),*)*
        }

        impl $struct_name {
            pub(crate) fn new(data: $data) -> Box<$struct_name> {
                use $crate::wlroots_sys::server::WAYLAND_SERVER_HANDLE;
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

            $($(pub(crate) unsafe extern "C" fn $listener(&mut self)
                                                   -> *mut $crate::wlroots_sys::wl_listener {
                &mut self.$listener as *mut _
            })*)*

            $($(pub(crate) unsafe extern "C" fn $listener_func(listener:
                                                        *mut $crate::wlroots_sys::wl_listener,
                                                        data: *mut $crate::libc::c_void) {
                let manager: &mut $struct_name = &mut (*container_of!(listener,
                                                                      $struct_name,
                                                                      $listener));
                $crate::utils::handle_unwind(
                    ::std::panic::catch_unwind(
                        ::std::panic::AssertUnwindSafe(|| {
                            (|$($func_arg: $func_type,)*| { $body })(manager, data)
                        })));
            })*)*
        }
    }
}

macro_rules! wayland_listener_static {
    (static mut $static_manager: ident;
     $(($manager: ident, $builder: ident):
       $([
           $(
               $([$($extra_callback_name: ident: $extra_callback_type: ty),+])*
               ($fn_type: ty, $listener: ident, $builder_func: ident) => ($notify: ident, $callback: ident):
               |$($func_arg: ident: $func_type: ty,)*| unsafe $body: block;
           )*
       ])+
     )+
    ) => {
        $(
            #[derive(Default)]
            #[allow(dead_code)]
            /// A builder of static functions to manage and create resources.
            ///
            /// Implement the functions with the necessary signature, pass them
            /// to the builder, and then give the builder to the necessary
            /// structure in order to utilize them (usually it's `compositor::Builder`).
            pub struct $builder {
                $($(pub(crate) $callback: ::std::option::Option<$fn_type>,)*
                  $($($($extra_callback_name: ::std::option::Option<$extra_callback_type>,)*)*)*)*
            }

            impl $builder {
                $($(
                    /// Uses the provided callback as the receiver for the
                    /// event the type signature describes.
                    pub fn $builder_func(mut self, $callback: $fn_type) -> Self {
                        self.$callback = ::std::option::Option::Some($callback);
                        self
                    }
                    $($(
                        /// Uses the provided callback as the receiver for the
                        /// event the type signature describes.
                        pub fn $extra_callback_name(mut self, $extra_callback_name: $extra_callback_type)
                                                    -> Self {
                            self.$extra_callback_name = ::std::option::Option::Some($extra_callback_name);
                            self
                        }
                    )*)*
                )*)*
            }
        )*

        $(
            #[repr(C)]
            pub(crate) struct $manager {
                $($(
                    pub(crate) $listener: $crate::wlroots_sys::wl_listener,
                    $callback: ::std::option::Option<$fn_type>,
                    $($($extra_callback_name: ::std::option::Option<$extra_callback_type>),*)*
                )*)*
            }

            pub(crate) static mut $static_manager: $manager = $manager {
                $($(
                    $listener: $crate::wlroots_sys::wl_listener {
                        link: {
                            $crate::wlroots_sys::wl_list {
                                prev: ::std::ptr::null_mut(),
                                next: ::std::ptr::null_mut()}},
                        notify: ::std::option::Option::None },
                    $callback: ::std::option::Option::None,
                    $($($extra_callback_name: ::std::option::Option::None),*)*
                )*)*
            };

            impl $manager {
                /// Sets the functions on the builder as the global manager functions.
                ///
                /// # Safety
                /// Returns a mutable reference to static data, which is unsafe to have
                /// multiple of. Do all your mutation through this reference and don't
                /// call this function multiple times.
                pub(crate) unsafe fn build(builder: $builder) -> &'static mut $manager {
                    $($(
                        $static_manager.$listener = {
                            // NOTE Rationale for zeroed memory:
                            // * Need to pass a pointer to wl_list_init
                            // * The list is initialized by Wayland, which doesn't "drop"
                            // * The listener is written to without dropping any of the data
                            let mut listener: $crate::wlroots_sys::wl_listener = ::std::mem::zeroed();
                            use $crate::wlroots_sys::server::WAYLAND_SERVER_HANDLE;
                            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                          wl_list_init,
                                          &mut listener.link as *mut _ as _);
                            ::std::ptr::write(&mut listener.notify, std::option::Option::Some($notify));
                            listener
                        };
                        $static_manager.$callback = builder.$callback;
                        $($(
                            $static_manager.$extra_callback_name = builder.$extra_callback_name;
                        )*)*
                    )*)*
                    &mut $static_manager
                }
            }
        )*


        $(
            $(
                $(
                    unsafe extern "C" fn $notify(listener: *mut $crate::wlroots_sys::wl_listener,
                                                 data: *mut $crate::libc::c_void) {
                        let manager: &mut $manager = &mut *container_of!(listener,
                                                                         $manager,
                                                                         $listener);
                        $crate::utils::handle_unwind(
                            ::std::panic::catch_unwind(
                                ::std::panic::AssertUnwindSafe(|| {
                                    (|$($func_arg: $func_type,)*| { $body })(manager, data)
                                })))
                    }
                )*
            )*
        )*
    }
}

/// A convenience macro designed for use with Handle types.
///
/// This allows you to avoid the rightward drift of death that is often found
/// with heavily nested callback systems.
///
/// Any `HandleResult`s are flattened and the first one encountered is
/// immediately returned before any of the `$body` code is executed.
///
/// Order of evaluation is from left to right. It is possible to refer to the
/// previous result, as commonly found in Lisp's `let*` macro.
///
/// An example of simple use:
///
/// ```rust,ignore
/// with_handles!([(compositor: {compositor}),
///    (output: {&mut result.output_handle})] => {
///    ...
/// })
/// ```
///
/// A more complex use:
///
/// ```rust,ignore
/// with_handles!([(shell: {shell_handle}),
///    // Notice how we use the previous result to get the surface.
///    (surface: {shell.surface_handle})] => {
///    ...
/// })
/// ```
#[cfg(feature = "unstable")]
#[macro_export]
macro_rules! with_handles {
    ([($handle_name: ident: $unhandle_name: block)] => $body: block) => {
        $unhandle_name.run(|$handle_name| {
            $body
        })
    };
    ([($handle_name1: ident: $unhandle_name1: block),
      ($handle_name2: ident: $unhandle_name2: block),
      $($rest: tt)*] => $body: block) => {
        $unhandle_name1.run(|$handle_name1| {
            with_handles!([($handle_name2: $unhandle_name2), $($rest)*] => $body)
        }).and_then(|n: $crate::utils::HandleResult<_>| n)
    };
    ([($handle_name: ident: $unhandle_name: block), $($rest: tt)*] => $body: block) => {
        $unhandle_name.run(|$handle_name| {
            with_handles!([$($rest)*] => $body)
        }).and_then(|n: $crate::utils::HandleResult<_>| n)
    };
}
