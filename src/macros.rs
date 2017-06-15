//! Macros for wlroots-rs
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
    &(*(0 as *const $ty)).$field as *const _ as usize
}
}


macro_rules! wl_container_of {
    ($ptr:ident, $ty:ty, $field:ident) => {
        ($ptr as usize - offset_of!($ty, $field)) as *const $ty
    }
}
