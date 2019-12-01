extern crate gl;
extern crate image;

use std::convert::TryInto;
use std::ffi::c_void;

use image::*;

pub struct Texture2D {
    id: gl::types::GLuint,
}

impl Texture2D {
    pub fn new_from_image(filename: &str) -> Result<Self, &'static str> {
        let mut t = Texture2D { id: 0 };
        unsafe {
            gl::GenTextures(1, &mut t.id);
        };
        self.bind();

        let image = open(filename).unwrap();
        match image {
            ImageRgb8(img) => unsafe {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB as i32,
                    img.width() as i32,
                    img.height() as i32,
                    0,
                    gl::BGR,
                    gl::UNSIGNED_BYTE,
                    img.as_ptr() as *const std::ffi::c_void,
                );
            },
            ImageRgba8(img) => unsafe {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    img.width() as i32,
                    img.height() as i32,
                    0,
                    gl::BGRA,
                    gl::UNSIGNED_BYTE,
                    img.as_ptr() as *const std::ffi::c_void,
                );
            }
            _ => unimplemented!(),
        }

        unsafe {
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::REPEAT.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::REPEAT.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR.try_into().unwrap(),
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        t
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
