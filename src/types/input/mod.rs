mod input_device;
pub mod keyboard;
pub mod pointer;
pub mod switch;
pub mod touch;
pub mod tablet_tool;
pub mod tablet_pad;

pub use self::input_device::*;

pub mod manager {
    //! Input resources are managed by the input resource manager.
    //!
    //! To manage a particular type of input resource implement a function
    //! with the signature of its corresponding name. For example, to manage
    //! keyboards implement [`KeyboardAdded`](./type.KeyboardAdded.html).
    //!
    //! Pass those functions to an [`input::Builder`](./struct.Builder.html)
    //! which is then given to a `compositor::Builder`.
    pub use manager::input_manager::*;
}
