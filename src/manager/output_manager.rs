//! Manager that is called when an output is created or destroyed.

use std::{marker::PhantomData, panic, ptr};

use libc;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::wlr_output;

use {compositor,
     output::{self, Output, OutputState, UserOutput},
     utils::{self, Handleable}};


/// Used to ensure the output sets the mode before doing any other
/// operation on the Output.
pub struct Builder<'output> {
    output: output::Handle,
    phantom: PhantomData<&'output Output>
}

/// Used to ensure that the builder is used to construct
/// the output::Handler instance.
pub struct BuilderResult<'output> {
    pub output: output::Handle,
    result: Box<output::Handler>,
    phantom: PhantomData<&'output Output>
}

/// Wrapper around Output destruction so that you can't call
/// unsafe methods (e.g anything like setting the mode).
pub struct Destroyed(output::Handle);

impl<'output> Builder<'output> {
    /// Build the output with the best mode.
    ///
    /// To complete construction, return this in your implementation of
    /// `output::ManagerHandler::output_added`.
    pub fn build_best_mode<T: output::Handler + 'static>(mut self,
                                                       data: T)
                                                       -> BuilderResult<'output> {
        with_handles!([(output: {&mut self.output})] => {
            output.choose_best_mode();
        }).expect("Output was borrowed");
        BuilderResult { output: self.output,
                        result: Box::new(data),
                        phantom: PhantomData }
    }
}

impl Destroyed {
    // TODO Functions which are safe to use
}

#[derive(Default)]
pub struct ManagerBuilder {
    add_callback: Option<fn(compositor_handle: compositor::Handle,
                            output_builder: Builder)
                            -> Option<BuilderResult>>
}

impl ManagerBuilder {
    pub fn output_added(mut self,
                        add_callback: fn(compositor_handle: compositor::Handle,
                                         output_builder: Builder)
                                         -> Option<BuilderResult>) -> Self {
        self.add_callback = Some(add_callback);
        self
    }
}

#[repr(C)]
pub(crate) struct Manager {
    pub(crate) add_listener: wlroots_sys::wl_listener,
    add_callback: Option<fn(compositor_handle: compositor::Handle,
                            output_builder: Builder)
                            -> Option<BuilderResult>>
}

pub(crate) static mut MANAGER: Manager = Manager {
    add_listener: wlroots_sys::wl_listener {
        link: { wlroots_sys::wl_list { prev: ptr::null_mut(), next: ptr::null_mut()}},
        notify: None },
    add_callback: None
};

impl Manager {
    /// Sets the functions on the builder as the global manager functions.
    pub(crate) fn build(builder: ManagerBuilder) {
        unsafe {
            MANAGER.add_listener = {
                // NOTE Rationale for zeroed memory:
                // * Need to pass a pointer to wl_list_init
                // * The list is initialized by Wayland, which doesn't "drop"
                // * The listener is written to without dropping any of the data
                let mut listener: wlroots_sys::wl_listener = ::std::mem::zeroed();
                use wlroots_sys::server::WAYLAND_SERVER_HANDLE;
                ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                              wl_list_init,
                              &mut listener.link as *mut _ as _);
                ::std::ptr::write(&mut listener.notify, Some(add_notify));
                listener
            };
            MANAGER.add_callback = builder.add_callback;
        }
    }
}

unsafe extern "C" fn add_notify(listener: *mut wlroots_sys::wl_listener, data: *mut libc::c_void) {
    utils::handle_unwind(
        panic::catch_unwind(
            panic::AssertUnwindSafe(|| {
                let manager = &mut *container_of!(listener, Manager, add_listener);
                let data = data as *mut wlr_output;
                let output = Output::new(data as *mut wlr_output);
                // NOTE
                // This clone is required because we pass it mutably to the output builder,
                // but due to lack of NLL there's no way to tell Rust it's safe to use it in
                // in the if branch.
                //
                // Thus, we need to clone it here and then drop the original once at the end.
                //
                // This is not a real clone, but an pub(crate) unsafe one we added, so it doesn't
                // break safety concerns in user code. Just an unfortunate hack we have to put here.
                let output_clone = output.clone();
                let builder = Builder { output: output.weak_reference(), phantom: PhantomData };
                let compositor = match compositor::handle() {
                    Some(handle) => handle,
                    None => return
                };
                let res = panic::catch_unwind(
                    panic::AssertUnwindSafe(|| manager.add_callback
                                            .map(|f| f(compositor, builder))
                                            .unwrap_or(None)));
                let build_result = match res {
                    Ok(res) => res,
                    // NOTE
                    // Either Wayland or wlroots does not handle failure to set up output correctly.
                    // Calling wl_display_terminate does not work if output is incorrectly set up.
                    //
                    // Instead, execution keeps going with an eventual segfault (if lucky).
                    //
                    // To fix this, we abort the process if there was a panic in output setup.
                    Err(_) => ::std::process::abort()
                };
                if let Some(BuilderResult {result: output_ptr, .. }) = build_result {
                    let mut output = UserOutput::new((output_clone, output_ptr));
                    wl_signal_add(&mut (*data).events.frame as *mut _ as _,
                                  output.frame_listener() as _);
                    wl_signal_add(&mut (*data).events.mode as *mut _ as _,
                                  output.mode_listener() as _);
                    wl_signal_add(&mut (*data).events.enable as *mut _ as _,
                                  output.enable_listener() as _);
                    wl_signal_add(&mut (*data).events.scale as *mut _ as _,
                                  output.scale_listener() as _);
                    wl_signal_add(&mut (*data).events.transform as *mut _ as _,
                                  output.transform_listener() as _);
                    wl_signal_add(&mut (*data).events.swap_buffers as *mut _ as _,
                                  output.swap_buffers_listener() as _);
                    wl_signal_add(&mut (*data).events.needs_swap as *mut _ as _,
                                  output.need_swap_listener() as _);
                    wl_signal_add(&mut (*data).events.destroy as *mut _ as _,
                                  output.on_destroy_listener() as _);
                    let output_data = (*data).data as *mut OutputState;
                    (*output_data).output = Box::into_raw(output);
                }
            })));
}
