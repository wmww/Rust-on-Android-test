extern crate gl;

use std;

pub struct Texture {
    id: gl::types::GLuint,
}

impl Texture {
    pub fn new() -> Result<Texture, String> {
        unsafe {
            let mut id = std::mem::uninitialized();
            gl::GenTextures(1, &mut id);
            Ok(Texture { id: id })
        }
    }

    pub fn bind_then<F>(&self, mut operation: F)
    where
        F: FnMut(),
    {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
        operation();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
