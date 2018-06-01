extern crate gl;

use std;
use std::mem;

use gl_basic;

pub type Float = gl::types::GLfloat;

#[repr(packed)]
pub struct Vec2 {
    pub x: gl::types::GLfloat,
    pub y: gl::types::GLfloat,
}

#[repr(packed)]
pub struct Vec3 {
    pub x: gl::types::GLfloat,
    pub y: gl::types::GLfloat,
    pub z: gl::types::GLfloat,
}

#[repr(packed)]
pub struct Vec4 {
    pub x: gl::types::GLfloat,
    pub y: gl::types::GLfloat,
    pub z: gl::types::GLfloat,
    pub w: gl::types::GLfloat,
}

trait GlType {
    fn gl_type() -> gl::types::GLenum;
    fn element_count() -> gl::types::GLint;
}

fn byte_size_of_gl_type(gl_type: gl::types::GLenum) -> gl::types::GLint {
    match gl_type {
        gl::FLOAT => mem::size_of::<gl::types::GLfloat>() as gl::types::GLint,
        t => panic!("unknown GL type {}", t),
    }
}

impl GlType for Float {
    fn gl_type() -> gl::types::GLenum { gl::FLOAT }
    fn element_count() -> gl::types::GLint { 1 }
}

impl GlType for Vec2 {
    fn gl_type() -> gl::types::GLenum { gl::FLOAT }
    fn element_count() -> gl::types::GLint { 2 }
}

impl GlType for Vec3 {
    fn gl_type() -> gl::types::GLenum { gl::FLOAT }
    fn element_count() -> gl::types::GLint { 3 }
}

impl GlType for Vec4 {
    fn gl_type() -> gl::types::GLenum { gl::FLOAT }
    fn element_count() -> gl::types::GLint { 4 }
}

struct Attrib {
    name: String,
    gl_type: gl::types::GLenum,
    element_count: gl::types::GLint,
}

impl Attrib {
    fn byte_size(&self) -> gl::types::GLint {
        byte_size_of_gl_type(self.gl_type) * self.element_count
    }
}

#[repr(packed)]
pub struct VertexData {
    pub point: Vec2,
    pub tex: Vec2,
    pub color: Vec4,
}

pub struct Object {
    program: gl_basic::Program,
    vertex_array_id: gl::types::GLuint,
    vertex_buffer_id: gl::types::GLuint,
    element_buffer_id: gl::types::GLuint,
    tri_count: u32,
}

impl Object {
    pub fn new(program: gl_basic::Program) -> Result<Object, String> {
        unsafe {
            let mut vertex_array = mem::uninitialized();
            gl::GenVertexArrays(1, &mut vertex_array);

            let mut vertex_buffer = mem::uninitialized();
		    gl::GenBuffers(1, &mut vertex_buffer);

            let mut element_buffer = mem::uninitialized();
            gl::GenBuffers(1, &mut element_buffer);

            gl::BindVertexArray(vertex_array);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer);
            //remember: do NOT unbind element_buffer, keep it bound to this vertex array

            // prevent bugs
            gl::BindVertexArray(0);

            Ok(Object {
                    program: program,
                    vertex_array_id: vertex_array,
                    vertex_buffer_id: vertex_buffer,
                    element_buffer_id: element_buffer,
                    tri_count: 0,
                })
        }
    }

    pub fn set_attribs_default(&mut self) {
        self.set_attribs(vec![
            Attrib{name: "position".to_string(), gl_type: Vec2::gl_type(), element_count: Vec2::element_count()},
            Attrib{name: "tex_coords".to_string(), gl_type: Vec2::gl_type(), element_count: Vec2::element_count()},
            Attrib{name: "color".to_string(), gl_type: Vec4::gl_type(), element_count: Vec4::element_count()},
        ]);
    }

    pub fn set_attribs(&mut self, attribs: Vec<Attrib>) {
        unsafe {
            gl::BindVertexArray(self.vertex_array_id);
            let stride = attribs.iter().fold(0, |sum, attrib| sum + attrib.byte_size());
            let mut offset = 0;
            for attrib in attribs {
                let pos_attrib = gl::GetAttribLocation(
                    self.program.id,
                    std::ffi::CString::new(attrib.name.clone()).unwrap().as_ptr() as *const _);
                gl::VertexAttribPointer(pos_attrib as gl::types::GLuint,
                                        attrib.element_count, gl::FLOAT,
                                        gl::FALSE,
                                        stride as gl::types::GLsizei,
                                        offset as *const () as *const _);
                offset += attrib.byte_size();
                gl::EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            }
            gl::BindVertexArray(0);
        }
    }

    pub fn set_vertices(&mut self, data: Vec<VertexData>) {
        unsafe {
            gl::BindVertexArray(self.vertex_array_id);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (data.len() * mem::size_of::<VertexData>()) as gl::types::GLsizeiptr,
                           data.as_ptr() as *const _,
                           gl::DYNAMIC_DRAW);
            gl::BindVertexArray(0);
        }
    }

    pub fn set_indices(&mut self, data: Vec<[gl::types::GLuint; 3]>) {
        unsafe {
            gl::BindVertexArray(self.vertex_array_id);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (data.len() * 3 * mem::size_of::<gl::types::GLuint>()) as isize,
                           data.as_ptr() as *const _,
                           gl::DYNAMIC_DRAW);
            self.tri_count = data.len() as u32;
            gl::BindVertexArray(0);
        }
    }

    pub fn draw(&mut self) {
        unsafe {
            self.program.begin_use();
            gl::BindVertexArray(self.vertex_array_id);
            gl::DrawElements(gl::TRIANGLES,
                             (self.tri_count * 3) as i32,
                             gl::UNSIGNED_INT,
                             std::ptr::null());
            gl::BindVertexArray(0);
            self.program.end_use();
        }
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vertex_buffer_id);
            gl::DeleteBuffers(1, &self.element_buffer_id);
            gl::DeleteVertexArrays(1, &self.vertex_array_id);
        }
    }
}

/*
impl Object {
    pub fn new(program: gl_basic::Program) -> Result<ObjectInternal, String> {
        unsafe {
            let mut vertex_array = mem::uninitialized();
            gl::GenVertexArrays(1, &mut vertex_array);

            let mut vertex_buffer = mem::uninitialized();
		    gl::GenBuffers(1, &mut vertex_buffer);

            let mut element_buffer = mem::uninitialized();
            gl::GenBuffers(1, &mut element_buffer);

            gl::BindVertexArray(vertex_array);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);

            {
                let pos_attrib = gl::GetAttribLocation(program.id, b"position\0".as_ptr() as *const _);
                gl::VertexAttribPointer(pos_attrib as gl::types::GLuint, 2, gl::FLOAT, 0,
                                        8 * mem::size_of::<f32>() as gl::types::GLsizei,
                                        (0 * mem::size_of::<f32>()) as *const () as *const _);
                gl::EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            }

            {
                let tex_coords_attrib = gl::GetAttribLocation(program.id, b"tex_coords\0".as_ptr() as *const _);
                gl::VertexAttribPointer(tex_coords_attrib as gl::types::GLuint, 2, gl::FLOAT, 0,
                                        8 * mem::size_of::<f32>() as gl::types::GLsizei,
                                        (2 * mem::size_of::<f32>()) as *const () as *const _);
                gl::EnableVertexAttribArray(tex_coords_attrib as gl::types::GLuint);
            }

            {
                let color_attrib = gl::GetAttribLocation(program.id, b"color\0".as_ptr() as *const _);
                gl::VertexAttribPointer(color_attrib as gl::types::GLuint, 4, gl::FLOAT, 0,
                                        8 * mem::size_of::<f32>() as gl::types::GLsizei,
                                        (4 * mem::size_of::<f32>()) as *const () as *const _);
                gl::EnableVertexAttribArray(color_attrib as gl::types::GLuint);
            }

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer);
            //remember: do NOT unbind element_buffer, keep it bound to this vertex array

            // prevent bugs
            gl::BindVertexArray(0);

            Ok(ObjectInternal {
                    program: program,
                    vertex_array_id: vertex_array,
                    vertex_buffer_id: vertex_buffer,
                    element_buffer_id: element_buffer,
                    tri_count: 0,
                })
        }
    }

    pub fn set_vertex_data(&self, data: Vec<VertexData>) {
        unsafe {
            gl::BindVertexArray(self.vertex_array_id);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (data.len() * mem::size_of::<VertexData>()) as gl::types::GLsizeiptr,
                           data.as_ptr() as *const _,
                           gl::DYNAMIC_DRAW);
            gl::BindVertexArray(0);
        }
    }

    pub fn set_indices(&mut self, data: Vec<[gl::types::GLuint; 3]>) {
        unsafe {
            gl::BindVertexArray(self.vertex_array_id);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (data.len() * 3 * mem::size_of::<gl::types::GLuint>()) as isize,
                           data.as_ptr() as *const _,
                           gl::DYNAMIC_DRAW);
            self.tri_count = data.len() as u32;
            gl::BindVertexArray(0);
        }
    }

    pub fn draw(&self) {
        unsafe {
            self.program.begin_use();
            gl::BindVertexArray(self.vertex_array_id);
            gl::DrawElements(gl::TRIANGLES,
                             (self.tri_count * 3) as i32,
                             gl::UNSIGNED_INT,
                             std::ptr::null());
            gl::BindVertexArray(0);
            self.program.end_use();
        }
    }
}*/