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
                break
            }
            { $body }
            $pos = container_of!((*$pos).$field.next, $container, $field);
        }
    }
}

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

/// Logs a message using wlroots' logging capability.
///
/// Possible values for `verb`:
///
/// * L_SILENT
/// * L_INFO
/// * L_DEBUG
/// * L_ERROR
#[macro_export]
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
///     InputManager,
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
    ($struct_name: ident, $data: ty, $([
        $($listener: ident => $listener_func: ident :
          |$($func_arg:ident: $func_type:ty,)*| unsafe $body: block;)*])+) => {
        #[repr(C)]
        pub struct $struct_name {
            data: $data,
            $($($listener: $crate::wlroots_sys::wl_listener),*)*
        }

        impl $struct_name {
            pub(crate) fn new(data: $data) -> Box<$struct_name> {
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

            $($(pub(crate) unsafe extern "C" fn $listener(&mut self)
                                                   -> *mut $crate::wlroots_sys::wl_listener {
                &mut self.$listener as *mut _
            })*)*

            $($(pub(crate) unsafe extern "C" fn $listener_func(listener:
                                                        *mut $crate::wlroots_sys::wl_listener,
                                                        data: *mut libc::c_void) {
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

/// Used to indicate what data is global compositor data.
/// It will automatically implement the CompositorData trait for the struct,
/// and also add a method to `Compositor` to unwrap the data from the fat
/// pointer.
#[macro_export]
macro_rules! compositor_data {
    ($struct_name: ty) => {
        impl<'a>::std::convert::From<&'a mut $crate::Compositor> for &'a mut $struct_name {
            fn from(compositor: &'a mut $crate::Compositor) -> &'a mut $struct_name {
                &mut *compositor.data.downcast_mut::<$struct_name>()
                    .unwrap_or_else(|| {
                        wlr_log!(L_ERROR, "Could not cast compositor state to {:#?}",
                                 stringify!($struct_name));
                        panic!("Could not cast compositor state to correct value")
                    })
            }
        }
    }
}

/// A convenience macro designed for use with Handle types.
///
/// This allows you to avoid the rightward drift of death that is often found
/// with heavily nested callback systems.
///
/// Any `HandleResult`s are flattened and the first one encountered is immediately
/// returned before any of the `$body` code is executed.
///
/// Order of evaluation is from left to right. It is possible to refer to the previous
/// result, as commonly found in Lisp's `let*` macro.
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
///
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
        }).and_then(|n: $crate::HandleResult<_>| n)
    };
    ([($handle_name: ident: $unhandle_name: block), $($rest: tt)*] => $body: block) => {
        $unhandle_name.run(|$handle_name| {
            with_handles!([$($rest)*] => $body)
        }).and_then(|n: $crate::HandleResult<_>| n)
    };
}

/// An even more convenient macro for use with Handle types.
///
/// This allows you to avoid rightward drift from having handles nested inside
/// each other.
///
/// Any `HandleResult`s are flattened and the first one encountered is immediantly
/// returned.
///
/// Note that unlike `with_handles` it is possible for some of the code to execute
/// before a `HandleErr` is returned. At each @ line there's the possibility
/// it will return an Error early.
///
/// To make this more clear, it is mandated that a ? is appended to each of these lines.
///
/// Here is some code using `with_handles`:
#[macro_export]
macro_rules! dehandle {
    (@$handle_name: ident = $unhandle_name: block?; $($rest: tt)+) => {
        with_handles!([($handle_name: $unhandle_name)] => {
            dehandle!($($rest)+)
        })
    };
    ($line: expr; $($rest: tt)*) => {
        {
            $line;
            dehandle!($($rest)*)
        }
    };
    ($line: stmt; $($rest: tt)*) => {
        {
            $line;
            dehandle!($($rest)*)
        }
    };
    ($line: expr) => {
        $line
    };
}
