extern crate gl;

use std;
use std::mem;

use gl_basic;
use gl_basic::types;
use gl_basic::types::GlType;

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

    pub fn set_attribs(&mut self, attribs: Vec<types::Attrib>) {
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

    pub unsafe fn set_vertices(&mut self, size: gl::types::GLsizeiptr, data: *const std::os::raw::c_void) {
        gl::BindVertexArray(self.vertex_array_id);
        gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::DYNAMIC_DRAW);
        gl::BindVertexArray(0);
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

#[repr(packed)]
pub struct VertexData {
    pub position: types::Vec2,
    pub tex_coords: types::Vec2,
    pub color: types::Vec4,
}

impl VertexData {
    pub fn new_object(program: gl_basic::Program) -> Result<Object, String> {
        let mut object = Object::new(program);
        if let Ok(ref mut object) = &mut object {
            object.set_attribs(vec![
                    types::Attrib{name: "position".to_string(), gl_type: types::Vec2::gl_type(), element_count: types::Vec2::element_count()},
                    types::Attrib{name: "tex_coords".to_string(), gl_type: types::Vec2::gl_type(), element_count: types::Vec2::element_count()},
                    types::Attrib{name: "color".to_string(), gl_type: types::Vec4::gl_type(), element_count: types::Vec4::element_count()},
                ]);
        }
        object
    }

    pub fn set_object_vertices(object: &mut Object, data: Vec<VertexData>) {
        unsafe {
            object.set_vertices(
                (data.len() * mem::size_of::<VertexData>()) as gl::types::GLsizeiptr,
                data.as_ptr() as *const _);
        }
    }
}
