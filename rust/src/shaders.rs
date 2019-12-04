extern crate cgmath;
extern crate gl;

use std::convert::TryInto;
use std::ffi::CString;
use std::fs;

use cgmath::*;

pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub fn new(
        vertex_shader_filename: &str,
        fragment_shader_filename: &str,
    ) -> Result<Self, &'static str> {
        let mut sh = Self { id: 0 };

        let vertex_sh = Shader::compile_shader_object(vertex_shader_filename, gl::VERTEX_SHADER)?;
        let fragment_sh =
            Shader::compile_shader_object(fragment_shader_filename, gl::FRAGMENT_SHADER)?;

        println!("Linking program");

        unsafe {
            sh.id = gl::CreateProgram();
            gl::AttachShader(sh.id, vertex_sh);
            gl::AttachShader(sh.id, fragment_sh);
            gl::LinkProgram(sh.id);
        }
        Shader::get_and_print_error_msg(sh.id);

        unsafe {
            gl::DetachShader(sh.id, vertex_sh);
            gl::DetachShader(sh.id, fragment_sh);
            gl::DeleteShader(vertex_sh);
            gl::DeleteShader(fragment_sh);
        }

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
        let id;
        unsafe {
            id = gl::CreateShader(shader_type);
            gl::ShaderSource(id, 1, &strptr, &strlen);
            gl::CompileShader(id);
        }
        Shader::get_and_print_error_msg(id);

        return Ok(id);
    }

    fn get_and_print_error_msg(id: gl::types::GLuint) {
        let mut infolog_len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut infolog_len);
        }

        if infolog_len > 0 {
            let msg = create_cstring_whitespace(infolog_len as usize);
            unsafe {
                gl::GetShaderInfoLog(
                    id,
                    infolog_len,
                    std::ptr::null_mut(),
                    msg.as_ptr() as *mut i8,
                );
            }
            println!("{}", msg.to_string_lossy().to_owned());
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    fn get_uniform_location(&self, name: &str) -> i32 {
        self.bind();
        let n = CString::new(name).unwrap();
        let location = unsafe { gl::GetUniformLocation(self.id, n.as_ptr()) };
        if location == -1 {
            println!("failed to locate uniform: {}", name);
        }
        location
    }

    pub fn set_uniform_1f(&self, name: &str, val: &f32) {
        let id = self.get_uniform_location(name);
        unsafe {
            gl::Uniform1f(id, *val);
        }
    }

    pub fn set_uniform_3f(&self, name: &str, val: &Vector3<f32>) {
        let id = self.get_uniform_location(name);
        unsafe {
            gl::Uniform3fv(id, 1, val.as_ptr());
        }
    }

    pub fn set_uniform_4f(&self, name: &str, val: &Vector4<f32>) {
        let id = self.get_uniform_location(name);
        unsafe {
            gl::Uniform4fv(id, 1, val.as_ptr());
        }
    }

    pub fn set_uniform_mat3f(&self, name: &str, val: &Matrix3<f32>) {
        let id = self.get_uniform_location(name);
        unsafe {
            gl::UniformMatrix3fv(id, 1, gl::FALSE, val.as_ptr());
        }
    }

    pub fn set_uniform_mat4f(&self, name: &str, val: &Matrix4<f32>) {
        let id = self.get_uniform_location(name);
        unsafe {
            gl::UniformMatrix4fv(id, 1, gl::FALSE, val.as_ptr());
        }
    }

    pub fn set_uniform_1i(&self, name: &str, val: &i32) {
        let id = self.get_uniform_location(name);
        unsafe {
            gl::Uniform1i(id, *val);
        }
    }

    pub fn set_texture_slot(&self, name: &str, val: &u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + *val);
        }
        self.set_uniform_1i(name, &(*val).try_into().unwrap());
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

fn create_cstring_whitespace(len: usize) -> CString {
    let mut buffer = Vec::<u8>::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len));
    unsafe { CString::from_vec_unchecked(buffer) }
}
