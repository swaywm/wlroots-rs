mod output;
pub mod layout;
mod mode;
mod cursor;
mod damage;

pub use self::cursor::*;
pub use self::damage::*;
pub use self::output::*;
pub use self::mode::*;

pub mod manager {
    //! Output resources are managed by the output resource manager.
    //!
    //! Using the [`OutputBuilder`](./struct.OutputBuilder.html) a [`BuilderResult`](./struct.BuilderResult.html) is constructed in a function
    //! conforming to the [`OutputAdded`](./type.OutputAdded.html) type signature. That function is passed
    //! to the [`output::Builder`](./struct.Builder.html) which is then given to the `compositor::Builder`.
    pub use manager::output_manager::*;
}
