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
        t.bind();

        let image = open(filename).map_err(|_| "failed to load image")?;

        match image {
            ImageRgb8(img) => unsafe {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB8 as i32,
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
                    gl::RGBA8 as i32,
                    img.width() as i32,
                    img.height() as i32,
                    0,
                    gl::BGRA,
                    gl::UNSIGNED_BYTE,
                    img.as_ptr() as *const std::ffi::c_void,
                );
            },
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
        Ok(t)
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

pub struct TextureCubeMap {
    id: gl::types::GLuint,
}

impl TextureCubeMap {
    fn setup_side(filename: &str, side: gl::types::GLenum) -> Result<(), &'static str> {
        let image = open(filename).map_err(|_| "failed to load image")?;
        match image {
            ImageRgb8(img) => unsafe {
                gl::TexImage2D(
                    side,
                    0,
                    gl::RGB8 as i32,
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
                    side,
                    0,
                    gl::RGBA8 as i32,
                    img.width() as i32,
                    img.height() as i32,
                    0,
                    gl::BGRA,
                    gl::UNSIGNED_BYTE,
                    img.as_ptr() as *const std::ffi::c_void,
                );
            },
            _ => unimplemented!(),
        }
        Ok(())
    }

    pub fn new_from_images(
        file_px: &str,
        file_nx: &str,
        file_py: &str,
        file_ny: &str,
        file_pz: &str,
        file_nz: &str,
    ) -> Result<Self, &'static str> {
        let mut t = TextureCubeMap { id: 0 };
        unsafe {
            gl::GenTextures(1, &mut t.id);
        };
        t.bind();

        TextureCubeMap::setup_side(file_px, gl::TEXTURE_CUBE_MAP_POSITIVE_X)?;
        TextureCubeMap::setup_side(file_nx, gl::TEXTURE_CUBE_MAP_NEGATIVE_X)?;
        TextureCubeMap::setup_side(file_py, gl::TEXTURE_CUBE_MAP_POSITIVE_Y)?;
        TextureCubeMap::setup_side(file_ny, gl::TEXTURE_CUBE_MAP_NEGATIVE_Y)?;
        TextureCubeMap::setup_side(file_pz, gl::TEXTURE_CUBE_MAP_POSITIVE_Z)?;
        TextureCubeMap::setup_side(file_nz, gl::TEXTURE_CUBE_MAP_NEGATIVE_Z)?;

        unsafe {
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR.try_into().unwrap(),
            );
        }
        Ok(t)
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.id);
        }
    }
}

impl Drop for TextureCubeMap {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
