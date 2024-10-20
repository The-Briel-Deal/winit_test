use std::ptr::{null, slice_from_raw_parts};

use crate::gl::{self, get_gl_string, Gl};

extern "system" fn handle_log(
    _source: u32,
    _gltype: u32,
    _id: u32,
    _severity: u32,
    length: i32,
    message: *const i8,
    _user_param: *mut std::ffi::c_void,
) {
    /* Idk what I'm doing here, I feel like I should be able to print out the message easier
     * but idc enough to figure it out bc it works. */
    let message = slice_from_raw_parts(message, length as usize);
    unsafe {
        let message = &*message;
        let mut new_message = vec![];

        for num in message {
            let unum = *num as u8;
            new_message.push(unum);
        }
        let new_message = String::from_utf8(new_message).unwrap();
        dbg!(new_message);
    }
}

pub fn setup_logging(gl: &Gl) {
    if let Some(renderer) = get_gl_string(gl, gl::RENDERER) {
        println!("Running on {}", renderer.to_string_lossy());
    }
    if let Some(version) = get_gl_string(gl, gl::VERSION) {
        println!("OpenGL Version {}", version.to_string_lossy());
    }
    if let Some(shaders_version) = get_gl_string(gl, gl::SHADING_LANGUAGE_VERSION) {
        println!("Shaders version on {}", shaders_version.to_string_lossy());
    }
    unsafe {
        gl.Enable(gl::DEBUG_OUTPUT);
        gl.DebugMessageCallback(Some(handle_log), null());
    };
}
