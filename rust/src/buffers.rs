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
        count: i32,
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
                count,
                gl::FLOAT,
                norm,
                stride * std::mem::size_of::<f32>() as i32,
                (offset * std::mem::size_of::<f32>() as i32) as *const std::ffi::c_void,
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

pub struct IndexBuffer {
    id: gl::types::GLuint,
    count: usize,
}

impl IndexBuffer {
    pub fn new_static(indices: &[u32]) -> Self {
        let mut ib = Self {
            id: 0,
            count: indices.len(),
        };
        unsafe {
            gl::GenBuffers(1, &mut ib.id);
        }
        ib.bind();
        unsafe {
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (ib.count() * std::mem::size_of::<u32>())
                    .try_into()
                    .unwrap(),
                indices.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );
        }
        ib
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}
