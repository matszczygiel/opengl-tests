extern crate gl;
extern crate image;

use std::convert::TryInto;
use std::ffi::c_void;

use image::*;

pub struct Texture2D {
    id: gl::types::GLuint,
}

fn load_texture_from_image(
    image: DynamicImage,
    texture_type: gl::types::GLenum,
) -> Result<(), String> {
    image.flipv();
    match image {
        ImageRgb8(img) => unsafe {
            gl::TexImage2D(
                texture_type,
                0,
                gl::RGB8 as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                img.into_raw().as_ptr() as *const std::ffi::c_void,
            );
        },
        ImageRgba8(img) => unsafe {
            gl::TexImage2D(
                texture_type,
                0,
                gl::RGBA8 as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.into_raw().as_ptr() as *const std::ffi::c_void,
            );
        },
        ImageLuma8(img) => unsafe {
            gl::TexImage2D(
                texture_type,
                0,
                gl::R8 as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                img.into_raw().as_ptr() as *const std::ffi::c_void,
            );
        },
        _ => return Err("Unknown image format".to_string()),
    }
    Ok(())
}

fn setup_texture_from_image(filename: &str, texture_type: gl::types::GLenum) -> Result<(), String> {
    let image = open(filename).map_err(|_| format!("failed to load image: {}", filename))?;
    load_texture_from_image(image, texture_type)?;
    Ok(())
}

impl Texture2D {
    pub fn new_from_image(filename: &str) -> Result<Self, String> {
        let mut t = Texture2D { id: 0 };
        unsafe {
            gl::GenTextures(1, &mut t.id);
        };
        t.bind();
        setup_texture_from_image(filename, gl::TEXTURE_2D)?;

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
    pub fn new_from_images(
        file_px: &str,
        file_nx: &str,
        file_py: &str,
        file_ny: &str,
        file_pz: &str,
        file_nz: &str,
    ) -> Result<Self, String> {
        let mut t = TextureCubeMap { id: 0 };
        unsafe {
            gl::GenTextures(1, &mut t.id);
        };
        t.bind();

        setup_texture_from_image(file_px, gl::TEXTURE_CUBE_MAP_POSITIVE_X)?;
        setup_texture_from_image(file_nx, gl::TEXTURE_CUBE_MAP_NEGATIVE_X)?;
        setup_texture_from_image(file_py, gl::TEXTURE_CUBE_MAP_POSITIVE_Y)?;
        setup_texture_from_image(file_ny, gl::TEXTURE_CUBE_MAP_NEGATIVE_Y)?;
        setup_texture_from_image(file_pz, gl::TEXTURE_CUBE_MAP_POSITIVE_Z)?;
        setup_texture_from_image(file_nz, gl::TEXTURE_CUBE_MAP_NEGATIVE_Z)?;

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
