use std::{
    ffi::{CStr, CString},
    os::raw::c_void,
};

use glam::{vec3, Mat4, Vec3};

use crate::{
    gl::{
        self,
        types::{GLfloat, GLuint},
        Gl,
    },
    helper::get_rand_angle,
    shader::{Shader, ShaderTrait},
};

struct Transform {
    rotation: GLfloat,
    scale: Vec3,
    translation: Vec3,
}

pub struct Mesh {
    program: Shader,
    vertex_buffer: VertexBuffer,
    transform: Transform,
    fov: f32,
    texture_blend: GLfloat,
}

struct VertexBuffer {
    vbo: u32,
    vao: u32,
    ebo: u32,
}

impl VertexBuffer {
    pub fn new(gl: &Gl, buffer: &[f32]) -> Self {
        let mut vertex_buffer = Self {
            vbo: 0,
            vao: 0,
            ebo: 0,
        };

        unsafe {
            gl.GenBuffers(1, &mut vertex_buffer.vbo);

            vertex_buffer.bind_vbo(gl);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of_val(buffer)) as gl::types::GLsizeiptr,
                buffer.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            vertex_buffer.unbind_vbo(gl);
        };

        unsafe {
            gl.GenVertexArrays(1, &mut vertex_buffer.vao);
        };

        unsafe {
            gl.GenBuffers(1, &mut vertex_buffer.ebo);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vertex_buffer.ebo);

            let ebo_indicies = [
                0, 1, 3, // Triangle One
                1, 2, 3, // Triangle Two
            ];

            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(&ebo_indicies) as isize,
                ebo_indicies.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
        };

        vertex_buffer
    }

    pub fn bind_vbo(&self, gl: &Gl) {
        unsafe { gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo) };
    }
    pub fn unbind_vbo(&self, gl: &Gl) {
        unsafe { gl.BindBuffer(gl::ARRAY_BUFFER, 0) };
    }
    pub fn bind_vao(&self, gl: &Gl) {
        unsafe { gl.BindVertexArray(self.vao) };
    }

    pub fn unbind_vao(&self, gl: &Gl) {
        unsafe { gl.BindVertexArray(0) };
    }
    pub fn set_float_attribute_position(
        &self,
        gl: &Gl,
        shader_attribute_name: &str,
        program: u32,
        start: u32,
        length: u32,
        stride: u32,
    ) {
        unsafe {
            let c_shader_attribute_name = CString::new(shader_attribute_name).unwrap();
            let attrib = gl.GetAttribLocation(
                program,
                c_shader_attribute_name.as_ptr() as *const gl::types::GLchar,
            );

            self.bind_vao(gl);
            gl.VertexAttribPointer(
                attrib as gl::types::GLuint,
                length as i32,
                gl::FLOAT,
                gl::FALSE,
                stride as i32 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (start as usize * size_of::<f32>()) as *const c_void,
            );
            self.unbind_vao(gl);
        }
    }

    pub fn vao(&self) -> GLuint {
        self.vao
    }
}

impl Mesh {
    pub fn new(gl: &Gl, program: &Shader, translation: Vec3) -> Self {
        let mesh = Mesh {
            program: program.clone(),
            vertex_buffer: VertexBuffer::new(gl, &VERTEX_DATA),
            transform: Transform {
                rotation: get_rand_angle(),
                translation,
                scale: vec3(1.0, 1.0, 1.0),
            },
            fov: 80.0,
            texture_blend: 0.2,
        };

        Self::point_attributes_to_buffer(gl, &mesh.vertex_buffer, program.get_id());

        mesh
    }

    pub fn adjust_blend(&mut self, percent: f32) {
        self.texture_blend = (self.texture_blend + percent).clamp(0.0, 1.0);
    }
    pub fn blend(&self) -> f32 {
        self.texture_blend
    }

    pub fn rotate_by(&mut self, degrees: GLfloat) {
        let transform = &mut self.transform;
        transform.rotation += degrees;
    }

    pub fn adjust_zoom(&mut self, degrees: GLfloat) {
        self.fov = (self.fov + degrees).clamp(5.0, 80.0);
    }

    pub fn vao(&self) -> GLuint {
        self.vertex_buffer.vao()
    }

    pub fn draw(&mut self, gl: &Gl, view_matrix: Mat4) {
        self.rotate_by(1.0);
        let transform = &self.transform;

        let model_matrix = Mat4::IDENTITY
            * Mat4::from_translation(transform.translation)
            * Mat4::from_rotation_x((transform.rotation / 2.0).to_radians())
            * Mat4::from_rotation_y(transform.rotation.to_radians())
            * Mat4::from_scale(transform.scale);

        let projection_matrix =
            Mat4::perspective_rh_gl(self.fov.to_radians(), gl.get_aspect_ratio(), 0.1, 100.0);

        self.program.set_mat4(gl, "model", model_matrix).unwrap();

        self.program.set_mat4(gl, "view", view_matrix).unwrap();

        self.program
            .set_mat4(gl, "projection", projection_matrix)
            .unwrap();

        unsafe {
            gl.BindVertexArray(self.vao());
            gl.DrawArrays(gl::TRIANGLES, 0, 36);
        }
    }

    fn point_attributes_to_buffer(gl: &gl::Gl, vbo: &VertexBuffer, program: u32) {
        unsafe {
            let pos_attrib =
                gl.GetAttribLocation(program, b"aPos\0".as_ptr() as *const gl::types::GLchar);
            let texture_coord_attrib =
                gl.GetAttribLocation(program, b"aTexCoord\0".as_ptr() as *const gl::types::GLchar);

            vbo.bind_vao(gl);
            gl.EnableVertexAttribArray(pos_attrib as GLuint);
            gl.EnableVertexAttribArray(texture_coord_attrib as GLuint);
            vbo.unbind_vao(gl);

            vbo.bind_vbo(gl);
            vbo.set_float_attribute_position(gl, "aPos", program, 0, 3, 5);
            vbo.set_float_attribute_position(gl, "aTexCoord", program, 3, 2, 5);
            vbo.unbind_vbo(gl);
        }
    }
}

#[rustfmt::skip]
static VERTEX_DATA: [f32; 180] = [
    // Vertices                      // Texture Coords
    -0.5_f32, -0.5_f32, -0.5_f32,    0.0_f32, 0.0_f32,
     0.5_f32, -0.5_f32, -0.5_f32,    1.0_f32, 0.0_f32,
     0.5_f32,  0.5_f32, -0.5_f32,    1.0_f32, 1.0_f32,
     0.5_f32,  0.5_f32, -0.5_f32,    1.0_f32, 1.0_f32,
    -0.5_f32,  0.5_f32, -0.5_f32,    0.0_f32, 1.0_f32,
    -0.5_f32, -0.5_f32, -0.5_f32,    0.0_f32, 0.0_f32,

    -0.5_f32, -0.5_f32,  0.5_f32,    0.0_f32, 0.0_f32,
     0.5_f32, -0.5_f32,  0.5_f32,    1.0_f32, 0.0_f32,
     0.5_f32,  0.5_f32,  0.5_f32,    1.0_f32, 1.0_f32,
     0.5_f32,  0.5_f32,  0.5_f32,    1.0_f32, 1.0_f32,
    -0.5_f32,  0.5_f32,  0.5_f32,    0.0_f32, 1.0_f32,
    -0.5_f32, -0.5_f32,  0.5_f32,    0.0_f32, 0.0_f32,

    -0.5_f32,  0.5_f32,  0.5_f32,    1.0_f32, 0.0_f32,
    -0.5_f32,  0.5_f32, -0.5_f32,    1.0_f32, 1.0_f32,
    -0.5_f32, -0.5_f32, -0.5_f32,    0.0_f32, 1.0_f32,
    -0.5_f32, -0.5_f32, -0.5_f32,    0.0_f32, 1.0_f32,
    -0.5_f32, -0.5_f32,  0.5_f32,    0.0_f32, 0.0_f32,
    -0.5_f32,  0.5_f32,  0.5_f32,    1.0_f32, 0.0_f32,

     0.5_f32,  0.5_f32,  0.5_f32,    1.0_f32, 0.0_f32,
     0.5_f32,  0.5_f32, -0.5_f32,    1.0_f32, 1.0_f32,
     0.5_f32, -0.5_f32, -0.5_f32,    0.0_f32, 1.0_f32,
     0.5_f32, -0.5_f32, -0.5_f32,    0.0_f32, 1.0_f32,
     0.5_f32, -0.5_f32,  0.5_f32,    0.0_f32, 0.0_f32,
     0.5_f32,  0.5_f32,  0.5_f32,    1.0_f32, 0.0_f32,

    -0.5_f32, -0.5_f32, -0.5_f32,    0.0_f32, 1.0_f32,
     0.5_f32, -0.5_f32, -0.5_f32,    1.0_f32, 1.0_f32,
     0.5_f32, -0.5_f32,  0.5_f32,    1.0_f32, 0.0_f32,
     0.5_f32, -0.5_f32,  0.5_f32,    1.0_f32, 0.0_f32,
    -0.5_f32, -0.5_f32,  0.5_f32,    0.0_f32, 0.0_f32,
    -0.5_f32, -0.5_f32, -0.5_f32,    0.0_f32, 1.0_f32,

    -0.5_f32,  0.5_f32, -0.5_f32,    0.0_f32, 1.0_f32,
     0.5_f32,  0.5_f32, -0.5_f32,    1.0_f32, 1.0_f32,
     0.5_f32,  0.5_f32,  0.5_f32,    1.0_f32, 0.0_f32,
     0.5_f32,  0.5_f32,  0.5_f32,    1.0_f32, 0.0_f32,
    -0.5_f32,  0.5_f32,  0.5_f32,    0.0_f32, 0.0_f32,
    -0.5_f32,  0.5_f32, -0.5_f32,    0.0_f32, 1.0_f32
];

#[cfg(test)]
mod test {
    use glam::{vec3, vec4, Mat4};

    #[test]
    fn test_translate() {
        let mut vec = vec4(1.0, 0.0, 0.0, 1.0);

        let trans = Mat4::from_translation(vec3(1.0, 1.0, 1.0));

        vec = trans * vec;

        assert_eq!(vec, vec4(2.0, 1.0, 1.0, 1.0));
    }
}
