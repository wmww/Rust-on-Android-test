extern crate gl;

use std;

const MAX_LOG_LENGTH: usize = 1024;

#[derive(PartialEq)]
pub enum ShaderType {
    Vert,
    Frag,
}

pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub fn new(shader_type: ShaderType, source: &str) -> Result<Shader, String> {
        unsafe {
            let id = gl::CreateShader(match &shader_type {
                    ShaderType::Vert => gl::VERTEX_SHADER,
                    ShaderType::Frag => gl::FRAGMENT_SHADER,
                });
            gl::ShaderSource(id, 1, [std::ffi::CString::new(source).unwrap().as_ptr() as *const _].as_ptr(), std::ptr::null());
            gl::CompileShader(id);

            // check for compile errors
            let mut success = gl::FALSE as i32;
            let mut info_log: Vec<u8> = Vec::with_capacity(MAX_LOG_LENGTH);
            info_log.set_len(MAX_LOG_LENGTH - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
            if success == gl::TRUE as i32 {
                Ok(Shader{id: id})
            }
            else
            {
                gl::GetShaderInfoLog(id, MAX_LOG_LENGTH as i32, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut i8);
                gl::DeleteShader(id);
                //match str::from_utf8_lossy(&info_log) {
                //    Ok(log) => log.to_string(),
                //    Err(error) => format!("filed to get log, error: {}", error),
                //});
                Err(format!("{} shader compile error: {}",
                    match &shader_type {
                            ShaderType::Vert => "vertex",
                            ShaderType::Frag => "fragment",
                        },
                    String::from_utf8_lossy(&info_log)))
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub struct Program {
    pub id: gl::types::GLuint,
}

impl Program {
    pub fn new(shaders: Vec<Shader>) -> Result<Program, String> {
        unsafe {
            let id = gl::CreateProgram();
            for shader in shaders {
                gl::AttachShader(id, shader.id);
            }
            gl::LinkProgram(id);

            // check for linking errors
            let mut success = gl::FALSE as i32;
            let mut info_log = Vec::with_capacity(MAX_LOG_LENGTH);
            info_log.set_len(MAX_LOG_LENGTH - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
            if success == gl::TRUE as i32 {
                Ok(Program{id: id})
            }
            else
            {
                gl::GetProgramInfoLog(id, MAX_LOG_LENGTH as i32, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut i8);
                gl::DeleteProgram(id);
                Err(format!("shader program link error: {}", String::from_utf8_lossy(&info_log)))
            }
        }
    }

    pub fn compile(vert_src: &str, frag_src: &str) -> Result<Program, String> {
        let vert = Shader::new(ShaderType::Vert, vert_src);
        let frag = Shader::new(ShaderType::Frag, frag_src);
        match (vert, frag) {
            (Ok(vert), Ok(frag)) => Program::new(vec![vert, frag]),
            (Err(vert_err), _) => Err(vert_err),
            (_, Err(frag_err)) => Err(frag_err),
        }
    }

    pub fn begin_use(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn end_use(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

/*
macro_rules! attribs {
    (struct $name:ident {
        $($field_name:ident: $field_type:ty,)*
    }) => {
        struct $name {
            $($field_name: $field_type,)*
        }

        impl $name {
            // This is purely an example—not a good one.
            fn get_field_names() -> Vec<&'static str> {
                vec![$(stringify!($field_name)),*]
            }
        }
    }
}
*/