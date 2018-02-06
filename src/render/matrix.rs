//! TODO Documentation

use wlroots_sys::{wl_output_transform, wlr_matrix_identity, wlr_matrix_mul, wlr_matrix_rotate,
                  wlr_matrix_scale, wlr_matrix_texture, wlr_matrix_transform, wlr_matrix_translate};

/// Modifies the matrix to become the identity matrix.
pub fn matrix_identity(output: &mut [f32; 16]) {
    unsafe { wlr_matrix_identity(output) }
}

/// Translate the matrix in the x, y, and z directions.
pub fn matrix_translate(output: &mut [f32; 16], x: f32, y: f32, z: f32) {
    unsafe { wlr_matrix_translate(output, x, y, z) }
}

/// Scale the output in the x, y, and z directions some amount.
pub fn matrix_scale(output: &mut [f32; 16], x: f32, y: f32, z: f32) {
    unsafe { wlr_matrix_scale(output, x, y, z) }
}

/// Rotate the matrix by some amount of radians.
pub fn matrix_rotate(output: &mut [f32; 16], radians: f32) {
    unsafe { wlr_matrix_rotate(output, radians) }
}

/// TODO Document
pub fn matrix_mul(x: &[f32; 16], y: &[f32; 16], product: &mut [f32; 16]) {
    unsafe { wlr_matrix_mul(x, y, product) }
}

/// Transform the matrix based on the given Wayland output transform mode.
pub fn matrix_transform(mat: &mut [f32; 16], transform: wl_output_transform) {
    unsafe { wlr_matrix_transform(mat.as_mut_ptr(), transform) }
}

/// Transform the matrix based on the given Wayland output transform mode and
/// the width and height of a texture.
pub fn matrix_texture(mat: &mut [f32; 16],
                      width: i32,
                      height: i32,
                      transform: wl_output_transform) {
    unsafe { wlr_matrix_texture(mat.as_mut_ptr(), width, height, transform) }
}
