//! TODO Documentation

use wlroots_sys::{wl_output_transform, wlr_matrix_identity, wlr_matrix_multiply, wlr_matrix_projection,
                  wlr_matrix_rotate, wlr_matrix_scale, wlr_matrix_transform, wlr_matrix_translate};

/// Modifies the matrix to become the identity matrix.
pub fn matrix_identity(output: &mut [f32; 16]) {
    unsafe { wlr_matrix_identity(output.as_mut_ptr()) }
}

/// Translate the matrix in the x, and y.
pub fn matrix_translate(output: &mut [f32; 16], x: f32, y: f32) {
    unsafe { wlr_matrix_translate(output.as_mut_ptr(), x, y) }
}

/// Scale the output in the x, and y.
pub fn matrix_scale(output: &mut [f32; 16], x: f32, y: f32) {
    unsafe { wlr_matrix_scale(output.as_mut_ptr(), x, y) }
}

/// Rotate the matrix by some amount of radians.
pub fn matrix_rotate(output: &mut [f32; 16], radians: f32) {
    unsafe { wlr_matrix_rotate(output.as_mut_ptr(), radians) }
}

/// TODO Document
pub fn matrix_mul(mat: &mut [f32; 16], x: [f32; 16], y: [f32; 16]) {
    unsafe { wlr_matrix_multiply(mat.as_mut_ptr(), x.as_ptr(), y.as_ptr()) }
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
    unsafe { wlr_matrix_projection(mat.as_mut_ptr(), width, height, transform) }
}
