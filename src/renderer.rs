use std::{borrow::Borrow, ffi::CString, ptr::null, rc::Rc};

use glam::vec3;
use glutin::prelude::GlDisplay;

use crate::{
    gl::{self, types::GLfloat, Gl},
    logging::setup_logging,
    mesh::Mesh,
    shader::{Shader, ShaderTrait},
};

pub struct Renderer {
    program: Shader,
    pub mesh_list: Vec<Mesh>,
    gl: Rc<Gl>,
}

impl Renderer {
    pub fn new<D: GlDisplay>(gl_display: &D) -> Self {
        let gl = Rc::new(gl::Gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        }));
        setup_logging(&gl);

        let program = Shader::new(gl.clone(), "src/shader/vert.glsl", "src/shader/frag.glsl");

        let mesh_list = vec![
            Mesh::new(gl.borrow(), &program, vec3(0.5, -0.5, 0.0)),
            Mesh::new(gl.borrow(), &program, vec3(0.5, 0.5, 0.0)),
        ];

        Self {
            program,
            mesh_list,
            gl,
        }
    }

    pub fn draw(&self) {
        self.draw_with_clear_color(0.1, 0.1, 0.1, 0.9)
    }

    pub fn draw_with_clear_color(
        &self,
        red: GLfloat,
        green: GLfloat,
        blue: GLfloat,
        alpha: GLfloat,
    ) {
        unsafe {
            let mesh1 = &self.mesh_list[0];
            let mesh2 = &self.mesh_list[1];


            self.gl.ClearColor(red, green, blue, alpha);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);

            self.program.enable();
            self.program
                .set_float("textureBlend", *mesh1.texture_blend.borrow())
                .unwrap();
            self.gl.ActiveTexture(gl::TEXTURE0);
            self.gl
                .BindTexture(gl::TEXTURE_2D, mesh1.get_texture("texture1"));
            self.gl.ActiveTexture(gl::TEXTURE1);
            self.gl
                .BindTexture(gl::TEXTURE_2D, mesh1.get_texture("texture2"));

            mesh1.draw(&self.gl);
            mesh2.draw(&self.gl);
        }
    }
    pub fn resize(&self, width: i32, height: i32) {
        unsafe { self.gl.Viewport(0, 0, width, height) }
    }
}
