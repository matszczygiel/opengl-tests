extern crate cgmath;
extern crate gl;
extern crate image;

use std::convert::TryInto;
use std::ffi::c_void;
use std::fs::File;
use std::io::BufReader;

use cgmath::*;
use image::*;

use crate::camera::*;
use crate::shaders::*;
use crate::utils::*;

pub struct Texture2D {
    id: gl::types::GLuint,
}

fn load_texture_from_image(
    image: DynamicImage,
    texture_type: gl::types::GLenum,
) -> Result<(), String> {
    let format = match image {
        ImageRgb8(_) => (gl::RGB8, gl::RGB),
        ImageRgba8(_) => (gl::RGBA8, gl::RGBA),
        ImageLuma8(_) => (gl::R8, gl::RED),
        _ => return Err("Unknown image format".to_string()),
    };
    unsafe {
        gl::TexImage2D(
            texture_type,
            0,
            format.0 as i32,
            image.width() as i32,
            image.height() as i32,
            0,
            format.1,
            gl::UNSIGNED_BYTE,
            image.raw_pixels().as_ptr() as *const std::ffi::c_void,
        );
    }
    Ok(())
}

fn setup_texture_from_image(filename: &str, texture_type: gl::types::GLenum) -> Result<(), String> {
    let image = open(filename).map_err(|_| format!("failed to load image: {}", filename))?;
    let image = image.flipv();
    load_texture_from_image(image, texture_type)?;
    Ok(())
}

fn setup_texture_from_hdr_file(filename: &str) -> Result<(u32, u32), String> {
    let file = File::open(filename).map_err(|_| format!("Cannot open: {}", filename))?;
    let format = ImageFormat::from_path(filename)
        .map_err(|_| format!("Cannot guess format of file: {}", filename))?;
    if format != ImageFormat::HDR {
        return Err(format!("File format of: {} is not HDR", filename));
    }

    let decoder = hdr::HDRDecoder::new(BufReader::new(file))
        .map_err(|_| format!("Cannot create HDRdecoder for: {}", filename))?;
    let meta = decoder.metadata();
    let buffer = {
        let data = decoder
            .read_image_hdr()
            .map_err(|_| format!("Cannot read file: {}", filename))?;
        let data_raw = data
            .into_iter()
            .map(|p| Vec::from(p.channels()))
            .flatten()
            .collect::<Vec<f32>>();
        let image_buf =
            ImageBuffer::<Rgb<f32>, Vec<f32>>::from_vec(meta.width, meta.height, data_raw)
                .ok_or("Failed to create image buffer")?;
        imageops::flip_vertical(&image_buf)
    };

    unsafe {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB16F as i32,
            meta.width as i32,
            meta.height as i32,
            0,
            gl::RGB,
            gl::FLOAT,
            buffer.into_raw().as_ptr() as *const std::ffi::c_void,
        );
    }
    Ok((meta.width, meta.height))
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
            gl::GenerateMipmap(gl::TEXTURE_2D);
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
        }
        Ok(t)
    }

    pub fn new_from_hdr(filename: &str) -> Result<(Self, u32, u32), String> {
        let mut t = Texture2D { id: 0 };
        unsafe {
            gl::GenTextures(1, &mut t.id);
        };
        t.bind();
        let (width, height) = setup_texture_from_hdr_file(filename)?;

        unsafe {
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR.try_into().unwrap(),
            );
        }
        Ok((t, width, height))
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
    pub fn set_slot(&self, val: &u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + *val);
        }
        self.bind();
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

lazy_static! {
    static ref CAPTURE_PERSPECTIVE: Matrix4<f32> = perspective(Deg(90.0), 1.0, 0.1, 10.0);
    static ref CAPTURE_VIEWS: [Matrix4<f32>; 6] = [
        Matrix4::look_at(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            vec3(0.0, -1.0, 0.0),
        ),
        Matrix4::look_at(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(-1.0, 0.0, 0.0),
            vec3(0.0, -1.0, 0.0),
        ),
        Matrix4::look_at(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            vec3(0.0, 0.0, 1.0),
        ),
        Matrix4::look_at(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, -1.0, 0.0),
            vec3(0.0, 0.0, -1.0),
        ),
        Matrix4::look_at(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            vec3(0.0, -1.0, 0.0),
        ),
        Matrix4::look_at(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, -1.0),
            vec3(0.0, -1.0, 0.0),
        ),
    ];
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

    pub fn new_from_hdr(filename: &str, face_resolution: i32) -> Result<Self, String> {
        let (hdr_texture, _, _) = Texture2D::new_from_hdr(filename)?;

        let mut capture_fbo = 0;
        let mut capture_rbo = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut capture_fbo);
            gl::GenRenderbuffers(1, &mut capture_rbo);

            gl::BindFramebuffer(gl::FRAMEBUFFER, capture_fbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, capture_rbo);
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH_COMPONENT24,
                face_resolution,
                face_resolution,
            );
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::RENDERBUFFER,
                capture_rbo,
            );
        }

        let mut env_cubemap = 0;
        unsafe {
            gl::GenTextures(1, &mut env_cubemap);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, env_cubemap);
        }
        for i in 0..6 {
            unsafe {
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                    0,
                    gl::RGB16F as i32,
                    face_resolution,
                    face_resolution,
                    0,
                    gl::RGB,
                    gl::FLOAT,
                    std::ptr::null(),
                );
            }
        }
        unsafe {
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as i32,
            );
        }

        let conversion_shader =
            Shader::new("../shaders/hdr_to_cube.vert", "../shaders/hdr_to_cube.frag").unwrap();

        conversion_shader.bind();
        conversion_shader.set_uniform_1i("equirectangular_map", &0);
        conversion_shader.set_uniform_mat4f("projection", &CAPTURE_PERSPECTIVE);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }
        hdr_texture.bind();
        unsafe {
            gl::Viewport(0, 0, face_resolution, face_resolution);
            gl::BindFramebuffer(gl::FRAMEBUFFER, capture_fbo);
        }

        let (va, _vb) = crate_cube_buffers();
        unsafe {
            gl::FrontFace(gl::CW);
        }
        for i in 0..CAPTURE_VIEWS.len() {
            conversion_shader.set_uniform_mat4f("view", &CAPTURE_VIEWS[i]);
            unsafe {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    env_cubemap,
                    0,
                );
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
            draw_cube(&va);
        }

        unsafe {
            gl::FrontFace(gl::CCW);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            gl::DeleteFramebuffers(1, &capture_fbo);
            gl::DeleteRenderbuffers(1, &capture_rbo);
        }
        let res = TextureCubeMap { id: env_cubemap };
        res.bind();
        unsafe {
            gl::GenerateMipmap(gl::TEXTURE_CUBE_MAP);
        }

        Ok(res)
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.id);
        }
    }

    pub fn set_slot(&self, val: &u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + *val);
        }
        self.bind();
    }
}

impl Drop for TextureCubeMap {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

pub fn compute_irradiance_map(hdr_enviromental_map: &TextureCubeMap) -> TextureCubeMap {
    let mut irradiance_map = TextureCubeMap { id: 0 };
    unsafe {
        gl::GenTextures(1, &mut irradiance_map.id);
    }
    irradiance_map.bind();
    const IRR_MAP_SIZE: i32 = 32;

    unsafe {
        for i in 0..6 {
            gl::TexImage2D(
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                0,
                gl::RGB16F as i32,
                IRR_MAP_SIZE,
                IRR_MAP_SIZE,
                0,
                gl::RGB,
                gl::FLOAT,
                std::ptr::null(),
            );
        }
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_R,
            gl::CLAMP_TO_EDGE as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as i32,
        );
    }

    let mut capture_fbo = 0;
    let mut capture_rbo = 0;
    unsafe {
        gl::GenFramebuffers(1, &mut capture_fbo);
        gl::GenRenderbuffers(1, &mut capture_rbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, capture_fbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, capture_rbo);
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT24,
            IRR_MAP_SIZE,
            IRR_MAP_SIZE,
        );
    }

    let irradiance_shader = Shader::new(
        "../shaders/hdr_to_cube.vert",
        "../shaders/irradiance_convolution.frag",
    )
    .unwrap();
    irradiance_shader.bind();
    irradiance_shader.set_uniform_1i("environmental_map", &0);
    hdr_enviromental_map.set_slot(&0);
    irradiance_shader.set_uniform_mat4f("projection", &CAPTURE_PERSPECTIVE);

    unsafe {
        gl::Viewport(0, 0, IRR_MAP_SIZE, IRR_MAP_SIZE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, capture_fbo);
        gl::FrontFace(gl::CW);
    }
    let (va, _vb) = crate_cube_buffers();
    for i in 0..CAPTURE_VIEWS.len() {
        irradiance_shader.set_uniform_mat4f("view", &CAPTURE_VIEWS[i]);
        unsafe {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                irradiance_map.id,
                0,
            );
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        irradiance_shader.bind();
        draw_cube(&va);
    }

    unsafe {
        gl::FrontFace(gl::CCW);
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::DeleteFramebuffers(1, &capture_fbo);
        gl::DeleteRenderbuffers(1, &capture_rbo);
    }

    irradiance_map
}

pub fn compute_prefiltered_env_map(
    hdr_enviromental_map: &TextureCubeMap,
    face_resolution: i32,
) -> TextureCubeMap {
    let mut prefiltered_env_map = TextureCubeMap { id: 0 };
    unsafe {
        gl::GenTextures(1, &mut prefiltered_env_map.id);
    }
    prefiltered_env_map.bind();

    const PREFILTERED_MAP_SIZE: i32 = 128;
    unsafe {
        for i in 0..6 {
            gl::TexImage2D(
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                0,
                gl::RGB16F as i32,
                PREFILTERED_MAP_SIZE,
                PREFILTERED_MAP_SIZE,
                0,
                gl::RGB,
                gl::FLOAT,
                std::ptr::null(),
            );
        }
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_R,
            gl::CLAMP_TO_EDGE as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as i32,
        );
        gl::GenerateMipmap(gl::TEXTURE_CUBE_MAP);
    }
    let shader = Shader::new(
        "../shaders/hdr_to_cube.vert",
        "../shaders/prefiltered_env_map.frag",
    )
    .unwrap();
    const ENV_MAP_SLOT: i32 = 0;
    shader.set_uniform_1i("environmental_map", &ENV_MAP_SLOT);
    hdr_enviromental_map.set_slot(&(ENV_MAP_SLOT as u32));
    shader.set_uniform_mat4f("projection", &CAPTURE_PERSPECTIVE);
    shader.set_uniform_1i("env_map_resolution", &face_resolution);

    let mut capture_fbo = 0;
    let mut capture_rbo = 0;
    unsafe {
        gl::GenFramebuffers(1, &mut capture_fbo);
        gl::GenRenderbuffers(1, &mut capture_rbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, capture_fbo);
    }
    const MAX_MPS_LEVELS: i32 = 5;
    for mip in 0..MAX_MPS_LEVELS {
        let mip_width = 128 / (2 as i32).pow(mip as u32);
        let mip_height = 128 / (2 as i32).pow(mip as u32);
        unsafe {
            gl::BindRenderbuffer(gl::RENDERBUFFER, capture_rbo);
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH_COMPONENT24,
                mip_width,
                mip_height,
            );
            gl::FrontFace(gl::CW);
            gl::Viewport(0, 0, mip_width, mip_height);
        }

        let roughness = mip as f32 / (MAX_MPS_LEVELS - 1) as f32;
        shader.set_uniform_1f("roughness", &roughness);
        let (va, _vb) = crate_cube_buffers();
        for i in 0..CAPTURE_VIEWS.len() {
            shader.set_uniform_mat4f("view", &CAPTURE_VIEWS[i]);
            unsafe {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    prefiltered_env_map.id,
                    mip,
                );
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
            draw_cube(&va);
        }
    }
    unsafe {
        gl::FrontFace(gl::CCW);
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::DeleteFramebuffers(1, &capture_fbo);
        gl::DeleteRenderbuffers(1, &capture_rbo);
    }

    prefiltered_env_map
}

pub fn compute_lut_texture(resolution: i32) -> Texture2D {
    let mut id = 0;
    unsafe {
        gl::GenTextures(1, &mut id);
    }
    let res = Texture2D { id };
    res.bind();
    unsafe {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RG16F as i32,
            resolution,
            resolution,
            0,
            gl::RG,
            gl::FLOAT,
            std::ptr::null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }

    let mut capture_fbo = 0;
    let mut capture_rbo = 0;
    unsafe {
        gl::GenFramebuffers(1, &mut capture_fbo);
        gl::GenRenderbuffers(1, &mut capture_rbo);

        gl::BindFramebuffer(gl::FRAMEBUFFER, capture_fbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, capture_rbo);
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT24,
            resolution,
            resolution,
        );
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            res.id,
            0,
        );
        gl::Viewport(0, 0, resolution, resolution);
    }

    let shader = Shader::new("../shaders/lut_texture.vert", "../shaders/lut_texture.frag").unwrap();
    shader.bind();
    let (va, _vb) = create_quad_buffers();
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
    draw_quad(&va);

    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::DeleteFramebuffers(1, &capture_fbo);
        gl::DeleteRenderbuffers(1, &capture_rbo);
    }
    res
}
