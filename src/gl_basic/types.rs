extern crate gl;

use std::mem;

pub type Float = gl::types::GLfloat;

#[repr(packed)]
pub struct Vec2 {
    pub x: Float,
    pub y: Float,
}

#[repr(packed)]
pub struct Vec3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

#[repr(packed)]
pub struct Vec4 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
    pub w: Float,
}

pub trait GlType {
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

pub struct Attrib {
    pub name: String,
    pub gl_type: gl::types::GLenum,
    pub element_count: gl::types::GLint,
}

impl Attrib {
    pub fn byte_size(&self) -> gl::types::GLint {
        byte_size_of_gl_type(self.gl_type) * self.element_count
    }
}