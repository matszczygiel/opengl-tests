extern crate gl;

use std::convert::TryInto;
use std::fs;
use std::ffi::CString;

pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub fn new() -> Result<Self, &'static str> {
        let mut sh = Self { id: 0 };
        Ok(sh)
    }

    fn compile_shader_object(
        filename: &str,
        shader_type: gl::types::GLenum,
    ) -> Result<gl::types::GLuint, &'static str> {
        println!("Compiling shader {}", filename);
        let source = fs::read_to_string(filename).expect("Cannot read shader source file.");

        let strptr: *const gl::types::GLchar = source.as_ptr() as *const i8;
        let strlen: gl::types::GLint = source.len().try_into().unwrap();
        let mut id;
        unsafe {
            id = gl::CreateShader(shader_type);
            gl::ShaderSource(id, 1, &strptr, &strlen);
        }

        let mut infolog_len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut infolog_len);
        }

        if infolog_len > 0 {
            let mut msg = CString::new(Vec::with_capacity(infolog_len as usize));
            gl::GetShaderInfoLog(id, infolog_len, std::ptr::null(), msg.into_raw());            
        }




        return Ok(id);
    }

    pub fn bind(&self) {
        gl::UseProgram(self.id);
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}
