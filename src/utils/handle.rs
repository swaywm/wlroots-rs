//! The generic implementation of a "handle" proxy object used throughout wlroots-rs.

use std::{clone::Clone, cell::Cell, error::Error, fmt, rc::Weak,
          hash::{Hash, Hasher}, ptr, panic, marker::PhantomData};

/// The result of trying to upgrade a handle, either using `run` or
/// `with_handles!`.
pub type HandleResult<T> = Result<T, HandleErr>;

/// The types of ways upgrading a handle can fail.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HandleErr {
    /// Attempting a handle that already has a mutable borrow to its
    /// backing structure.
    AlreadyBorrowed,
    /// Tried to upgrade a handle for a structure that has already been dropped.
    AlreadyDropped
}

/// A non-owned reference counted handle to a resource.
///
/// The resource could be destroyed at any time, it depends on the resource.
///
/// For example an output is destroyed when it's physical output is "disconnected"
/// on DRM. "disconnected" depends on the output (e.g. sometimes turning it off counts
/// as "disconnected").
/// However, when the backend is instead headless an output lives until it is
/// destroyed explicitly by the library user.
///
/// Some resources are completely controlled by the user. For example although
/// you refer to a `Seat` with handles it is only destroyed when you call the
/// special destroy method on the seat handle.
///
/// Please refer to the specific resource documentation for a description of
/// the lifetime particular to that resource.
pub struct Handle<D: Clone, T, W: Handleable<D, T> + Sized> {
    pub(crate) ptr: *mut T,
    pub(crate) handle: Weak<Cell<bool>>,
    pub(crate) _marker: PhantomData<W>,
    pub(crate) data: Option<D>
}

pub trait Handleable<D: Clone, T> {
    /// Constructs the resource manager from a raw pointer of the resource
    /// this handleable manages. **This should increment the reference count**.
    ///
    /// # Safety
    /// The pointer must be valid and must already have been set up by wlroots-rs
    /// internal mechanisms. If you have to ask, it probably isn't.
    #[doc(hidden)]
    unsafe fn from_ptr(resource_ptr: *mut T) -> Option<Self> where Self: Sized;

    /// Gets the pointer to the resource this object manages.
    #[doc(hidden)]
    unsafe fn as_ptr(&self) -> *mut T;

    /// Gets the resource from the handle.
    ///
    /// This is used internally to upgrade a handle, and should not be used.
    /// Thus the documentation is hidden and it's marked `unsafe`.
    ///
    /// If you _need_ to use this, use `Handle::upgrade` instead.
    #[doc(hidden)]
    unsafe fn from_handle(&Handle<D, T, Self>) -> HandleResult<Self> where Self: Sized;

    /// Creates a weak reference to the resource.
    fn weak_reference(&self) -> Handle<D, T, Self> where Self: Sized;
}

impl <D: Clone, T, W: Handleable<D, T>> Clone for Handle<D, T, W> {
    fn clone(&self) -> Self {
        Handle { ptr: self.ptr,
                 handle: self.handle.clone(),
                 _marker: PhantomData,
                 data: self.data.clone()
        }
    }
}

impl <D: Clone, T, W: Handleable<D, T>> fmt::Debug for Handle<D, T, W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Handle with pointer: {:p}", self.ptr)
    }
}

impl <D: Clone, T, W: Handleable<D, T>> Hash for Handle<D, T, W> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

impl <D: Clone, T, W: Handleable<D, T>> PartialEq for Handle<D, T, W> {
    fn eq(&self, other: &Handle<D, T, W>) -> bool {
        self.ptr == other.ptr
    }
}

impl <D: Clone, T, W: Handleable<D, T>> Eq for Handle<D, T, W> {}

impl <D: Clone, T, W: Handleable<D, T>> Default for Handle<D, T, W> {
    /// Constructs a new handle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    fn default() -> Self {
        Handle { ptr: ptr::null_mut(),
                 handle: Weak::new(),
                 _marker: PhantomData,
                 data: None }
    }
}

impl <D: Clone, T, W: Handleable<D, T>> Handle<D, T, W> {
    /// Creates an output::Handle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    ///
    /// # Panics
    /// This function is allowed to panic when attempting to upgrade the handle.
    #[allow(dead_code)]
    pub(crate) unsafe fn from_ptr(ptr: *mut T) -> Handle<D, T, W> {
        match W::from_ptr(ptr) {
            Some(wrapped_resource) => wrapped_resource.weak_reference(),
            None => {
                let mut handle = Self::default();
                handle.ptr = ptr;
                handle
            }
        }
    }

    /// Get the pointer to the resource this manages.
    ///
    /// # Safety
    /// There's no guarantees that this pointer is not dangling.
    #[doc(hidden)]
    pub unsafe fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    /// Run a function with a reference to the resource if it's still alive.
    ///
    /// Returns the result of the function, if successful.
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the resource
    /// to a short lived scope of an anonymous function,
    /// this function ensures the resource does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if a handle to the same resource was upgraded some where else up the stack.
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut W) -> R
    {
        let mut wrapped_obj = unsafe { self.upgrade()? };
        // We catch panics here to deal with an extreme edge case.
        //
        // If the library user catches panics from the `run` function then their
        // resource used flag will still be set to `true` when it should be set
        // to `false`.
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut wrapped_obj)));
        self.handle.upgrade().map(|check| {
            // Sanity check that it hasn't been tampered with. If so, we should just panic.
            // If we are currently panicking this will abort.
            if !check.get() {
                wlr_log!(WLR_ERROR, "After running callback, mutable lock was false");
                panic!("Lock in incorrect state!");
            }
            check.set(false);
        });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    /// Determines if the handle is alive or not.
    ///
    /// This does not check if it's already being borrowed.
    pub fn is_alive(&self) -> bool {
        self.handle.upgrade().map(|_| true).unwrap_or(false)
    }

    /// Determines if the handle is borrowed or not.
    ///
    /// If the handle is not alive it will return false.
    pub fn is_borrowed(&self) -> bool {
        self.handle.upgrade().map(|check| check.get()).unwrap_or(false)
    }

    /// Upgrades a handle to a reference to the backing object.
    ///
    /// # Safety
    /// This returns an "owned" value when really you don't own it all.
    /// Depending on the type, it's possible that the resource will be freed
    /// once this returned value is dropped, causing a possible double free.
    /// Potentially it instead is just unbound, it depends on the resource.
    ///
    /// Regardless, you should not use this interface. Use the `run` method.
    #[doc(hidden)]
    pub unsafe fn upgrade(&self) -> HandleResult<W> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                let wrapper_obj = W::from_handle(self)?;
                check.set(true);
                Ok(wrapper_obj)
            })

    }
}

impl fmt::Display for HandleErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::HandleErr::*;
        match *self {
            AlreadyBorrowed => write!(f, "already borrowed"),
            AlreadyDropped => write!(f, "already dropped")
        }
    }
}

impl Error for HandleErr {
    fn description(&self) -> &str {
        use self::HandleErr::*;
        match *self {
            AlreadyBorrowed => "Structure is already mutably borrowed",
            AlreadyDropped => "Structure has already been dropped"
        }
    }
}
