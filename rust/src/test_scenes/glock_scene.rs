extern crate cgmath;
extern crate gl;

use cgmath::*;

use super::*;
use crate::buffers::*;
use crate::camera::*;
use crate::shaders::*;
use crate::textures::*;
use crate::utils::*;

pub struct PbrGlock {
    skybox: (VertexArray, VertexBuffer, Shader, TextureCubeMap),
    glock: (VertexArray, VertexBuffer, IndexBuffer, Shader),
    ibl_setup: (TextureCubeMap, TextureCubeMap, Texture2D),
    glock_textures: (Texture2D, Texture2D, Texture2D, Texture2D, Texture2D),
    cam: Camera,
    moving_up: bool,
    moving_down: bool,
    moving_right: bool,
    moving_left: bool,
    framebuffer_size: (u32, u32),
}

impl PbrGlock {
    const ENV_MAP_FACE_RESOLUTION: i32 = 1024;
    const LUT_TEXTURE_RESOLUTION: i32 = 512;

    const CAM_SPEED: f32 = 0.00003;
    const FOV_SPEED: f32 = 1.05;
    const MOUSE_SPEED: f32 = 0.002;

    fn reload_shader(&mut self) {
        let shader = Shader::new(
            "../shaders/sphere_pbr.vert",
            "../shaders/sphere_textured_pbr_ibl.frag",
        )
        .unwrap();
        shader.set_uniform_1i("irradiance_map", &0);
        shader.set_uniform_1i("prefiltered_map", &1);
        shader.set_uniform_1i("brdf_lut", &2);

        shader.set_uniform_1i("albedo_map", &3);
        shader.set_uniform_1i("normal_map", &4);
        shader.set_uniform_1i("metallic_map", &5);
        shader.set_uniform_1i("roughness_map", &6);
        shader.set_uniform_1i("ao_map", &7);
        self.glock.3 = shader;
    }
}

impl TestScene for PbrGlock {
    fn new(framebuffer_size: (u32, u32)) -> Box<dyn TestScene> {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);

            gl::Enable(gl::CULL_FACE);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);
        }

        let skybox_texture = TextureCubeMap::new_from_hdr(
            "../resources/Factory_Catwalk/Factory_Catwalk_2k.hdr",
            Self::ENV_MAP_FACE_RESOLUTION,
        )
        .unwrap();

        let mut res = Box::new(Self {
            ibl_setup: {
                let irr = compute_irradiance_map(&skybox_texture);
                let pref =
                    compute_prefiltered_env_map(&skybox_texture, Self::ENV_MAP_FACE_RESOLUTION);

                let lut = compute_lut_texture(Self::LUT_TEXTURE_RESOLUTION);
                (irr, pref, lut)
            },
            glock_textures: {
                let generate_from_path = |path: &str| {
                    let albedo =
                        Texture2D::new_from_image(&format!("{}/albedo.png", path)).unwrap();
                    let normal =
                        Texture2D::new_from_image(&format!("{}/normal.png", path)).unwrap();
                    let metallic =
                        Texture2D::new_from_image(&format!("{}/metallic.png", path)).unwrap();
                    let roughness =
                        Texture2D::new_from_image(&format!("{}/roughness.png", path)).unwrap();
                    let ao = Texture2D::new_from_image(&format!("{}/ao.png", path)).unwrap();
                    (albedo, normal, metallic, roughness, ao)
                };

                generate_from_path("../resources/glock/textures")
            },
            skybox: {
                let (va, vb) = create_skybox_buffers();
                let shader =
                    Shader::new("../shaders/skybox.vert", "../shaders/skybox.frag").unwrap();

                (va, vb, shader, skybox_texture)
            },
            glock: {
                let (va, vb, ib) = load_model("../resources/glock/glock.obj").unwrap();
                let shader = Shader::new(
                    "../shaders/sphere_pbr.vert",
                    "../shaders/sphere_textured_pbr_ibl.frag",
                )
                .unwrap();
                shader.set_uniform_1i("irradiance_map", &0);
                shader.set_uniform_1i("prefiltered_map", &1);
                shader.set_uniform_1i("brdf_lut", &2);

                shader.set_uniform_1i("albedo_map", &3);
                shader.set_uniform_1i("normal_map", &4);
                shader.set_uniform_1i("metallic_map", &5);
                shader.set_uniform_1i("roughness_map", &6);
                shader.set_uniform_1i("ao_map", &7);

                (va, vb, ib, shader)
            },
            cam: Camera::new_default(0.0, 0.0),
            moving_up: false,
            moving_down: false,
            moving_right: false,
            moving_left: false,
            framebuffer_size,
        });
        unsafe {
            gl::Viewport(0, 0, framebuffer_size.0 as i32, framebuffer_size.1 as i32);
        }
        res.reset();
        res
    }

    fn reset(&mut self) {
        self.cam = Camera::new_default(
            self.framebuffer_size.0 as f32,
            self.framebuffer_size.1 as f32,
        );
        self.cam.position.z = 5.0;
        self.reload_shader();
    }

    fn handle_event(&mut self, event: &Event<()>, _: &mut ControlFlow) {
        match event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                    Some(VirtualKeyCode::Up) => {
                        self.moving_up = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }
                    Some(VirtualKeyCode::Down) => {
                        self.moving_down = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }
                    Some(VirtualKeyCode::Right) => {
                        self.moving_right = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }
                    Some(VirtualKeyCode::Left) => {
                        self.moving_left = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }
                    Some(VirtualKeyCode::W) => {
                        self.cam.perspective.fovy /= Self::FOV_SPEED;
                        if self.cam.perspective.fovy < Rad::from(Deg(15.0)) {
                            self.cam.perspective.fovy = Rad::from(Deg(15.0));
                        }
                    }
                    Some(VirtualKeyCode::S) => {
                        self.cam.perspective.fovy *= Self::FOV_SPEED;
                        if self.cam.perspective.fovy > Rad::from(Deg(100.0)) {
                            self.cam.perspective.fovy = Rad::from(Deg(100.0));
                        }
                    }
                    _ => (),
                },
                _ => (),
            },
            Event::DeviceEvent { ref event, .. } => match event {
                DeviceEvent::MouseMotion { delta: (x, y) } => {
                    self.cam.horizontal_angle -= Rad(*x as f32 * Self::MOUSE_SPEED);
                    self.cam.vertical_angle -= Rad(*y as f32 * Self::MOUSE_SPEED);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn update(&mut self, delta: Duration) {
        let mut vel = vec3(0.0, 0.0, 0.0);
        if self.moving_up {
            vel += self.cam.direction();
        }
        if self.moving_down {
            vel -= self.cam.direction();
        }
        if self.moving_right {
            vel += self.cam.right();
        }
        if self.moving_left {
            vel -= self.cam.right();
        }
        self.cam.position += vel * delta.as_micros() as f32 * Self::CAM_SPEED;
    }

    fn render(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        let (view, projection) = self.cam.to_vp();
        let cam_pos = self.cam.position.to_homogeneous().truncate();
        let sphere_shader = &self.glock.3;
        sphere_shader.bind();
        sphere_shader.set_uniform_mat4f("projection", &projection);
        sphere_shader.set_uniform_mat4f("view", &view);
        sphere_shader.set_uniform_3f("world_cam_posiiton", &cam_pos);
        self.ibl_setup.0.set_slot(&0);
        self.ibl_setup.1.set_slot(&1);
        self.ibl_setup.2.set_slot(&2);

        const ROWS: i32 = 7;
        const COLS: i32 = 7;
        const SPACING: f32 = 2.5;

        self.glock_textures.0.set_slot(&3);
        self.glock_textures.1.set_slot(&4);
        self.glock_textures.2.set_slot(&5);
        self.glock_textures.3.set_slot(&6);
        self.glock_textures.4.set_slot(&7);
        let model: Matrix4<f32> = Transform::one();
        sphere_shader.set_uniform_mat4f("model", &model);
        draw_model(&self.glock.2, &self.glock.0);

        let skybox_shader = &self.skybox.2;
        skybox_shader.bind();
        skybox_shader.set_uniform_mat4f("view", &view);
        skybox_shader.set_uniform_mat4f("projection", &projection);
        skybox_shader.set_texture_slot("skybox", &0);
        self.skybox.3.set_slot(&0);

        draw_skybox(&self.skybox.0);
    }

    fn set_framebuffer_size(&mut self, size: (u32, u32)) {
        self.framebuffer_size = size;

        unsafe {
            gl::Viewport(
                0,
                0,
                self.framebuffer_size.0 as i32,
                self.framebuffer_size.1 as i32,
            );
        }
    }
}

impl Drop for PbrGlock {
    fn drop(&mut self) {
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::BLEND);
            gl::Disable(gl::TEXTURE_CUBE_MAP_SEAMLESS);
        }
    }
}
