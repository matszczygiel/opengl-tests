extern crate cgmath;
extern crate gl;

use cgmath::*;

use super::*;
use crate::buffers::*;
use crate::camera::*;
use crate::shaders::*;
use crate::textures::*;
use crate::utils::*;

pub struct PbrSpheres {
    skybox: (VertexArray, VertexBuffer, Shader, TextureCubeMap),
    spheres: (VertexArray, VertexBuffer, IndexBuffer, Shader),
    pbr_setup: (TextureCubeMap, TextureCubeMap, Texture2D),
    cam: Camera,
    moving_up: bool,
    moving_down: bool,
    moving_right: bool,
    moving_left: bool,
    framebuffer_size: (u32, u32),
}

impl PbrSpheres {
    const ENV_MAP_FACE_RESOLUTION: i32 = 1024;
    const LUT_TEXTURE_RESOLUTION: i32 = 512;

    const CAM_SPEED: f32 = 0.00003;
    const FOV_SPEED: f32 = 1.05;
    const MOUSE_SPEED: f32 = 0.002;
}

impl TestScene for PbrSpheres {
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
            pbr_setup: (
                compute_irradiance_map(&skybox_texture),
                compute_prefiltered_env_map(&skybox_texture, Self::ENV_MAP_FACE_RESOLUTION),
                compute_lut_texture(Self::LUT_TEXTURE_RESOLUTION),
            ),
            skybox: {
                let (va, vb) = create_skybox_buffers();
                let shader =
                    Shader::new("../shaders/skybox.vert", "../shaders/skybox.frag").unwrap();

                (va, vb, shader, skybox_texture)
            },
            spheres: {
                let (va, vb, ib) = crate_sphere_buffers(1.0);
                let shader = Shader::new(
                    "../shaders/sphere_pbr.vert",
                    "../shaders/sphere_pbr_ibl.frag",
                )
                .unwrap();
                shader.set_uniform_3f("albedo", &vec3(0.5, 0.5, 0.5));
                shader.set_uniform_1f("ao", &1.0);
                shader.set_uniform_1i("irradiance_map", &0);
                shader.set_uniform_1i("prefiltered_map", &1);
                shader.set_uniform_1i("brdf_lut", &2);
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
        let sphere_shader = &self.spheres.3;
        sphere_shader.bind();
        sphere_shader.set_uniform_mat4f("projection", &projection);
        sphere_shader.set_uniform_mat4f("view", &view);
        sphere_shader.set_uniform_3f("world_cam_posiiton", &cam_pos);
        self.pbr_setup.0.set_slot(&0);
        self.pbr_setup.1.set_slot(&1);
        self.pbr_setup.2.set_slot(&2);

        const ROWS: i32 = 7;
        const COLS: i32 = 7;
        const SPACING: f32 = 2.5;

        for row in 0..ROWS {
            let metallness = row as f32 / ROWS as f32;
            sphere_shader.set_uniform_1f("metallic", &metallness);
            for col in 0..COLS {
                let roughness = (col as f32 / COLS as f32).max(0.05).min(1.0);
                sphere_shader.set_uniform_1f("roughness", &roughness);

                let translation = vec3::<f32>(
                    col as f32 - (COLS as f32 / 2.0),
                    row as f32 - (ROWS as f32 / 2.0),
                    0.0,
                ) * SPACING;
                let model = Matrix4::<f32>::from_translation(translation);
                sphere_shader.set_uniform_mat4f("model", &model);
                draw_sphere(&self.spheres.2, &self.spheres.0);
            }
        }

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

impl Drop for PbrSpheres {
    fn drop(&mut self) {
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::BLEND);
            gl::Disable(gl::TEXTURE_CUBE_MAP_SEAMLESS);
        }
    }
}
