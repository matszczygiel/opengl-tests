extern crate gl;

use std::convert::TryInto;

pub struct VertexBuffer {
    id: gl::types::GLuint,
}

impl VertexBuffer {
    pub fn new_static(vertices: &[f32]) -> Self {
        let mut vb = Self { id: 0 };
        unsafe {
            gl::GenBuffers(1, &mut vb.id);
        }
        vb.bind();
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>())
                    .try_into()
                    .unwrap(),
                vertices.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );
        }
        vb
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

pub struct VertexArray {
    id: gl::types::GLuint,
}

impl VertexArray {
    pub fn new() -> Self {
        let mut va = Self { id: 0 };
        unsafe {
            gl::GenVertexArrays(1, &mut va.id);
        }
        va.bind();
        va
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn set_vertex_attrib_array(
        &self,
        index: u32,
        size: i32,
        normalized: bool,
        stride: i32,
        offset: i32,
    ) {
        let norm = {
            if normalized {
                gl::TRUE
            } else {
                gl::FALSE
            }
        };
        self.bind();
        unsafe {
            gl::VertexAttribPointer(
                index,
                size,
                gl::FLOAT,
                norm,
                stride,
                offset as *const std::ffi::c_void,
            );
            gl::EnableVertexAttribArray(index);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}
