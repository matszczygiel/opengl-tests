#![allow(dead_code)]

extern crate cgmath;
extern crate gl;
extern crate glutin;
extern crate wavefront_obj;
#[macro_use]
extern crate lazy_static;

mod buffers;
mod camera;
mod shaders;
mod textures;
mod utils;

use buffers::*;
use camera::*;
use shaders::*;
use textures::*;
use utils::*;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::time::Instant;

use cgmath::*;
use glutin::*;

extern "system" fn debug_callback(
    _: gl::types::GLenum,
    gltype: gl::types::GLenum,
    _: gl::types::GLuint,
    severity: gl::types::GLenum,
    _: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    _: *mut core::ffi::c_void,
) {
    print!("GL CALLBACK: ");
    if gltype == gl::DEBUG_TYPE_ERROR {
        print!("** GL ERROR **");
    }

    unsafe {
        println!(
            " type = {0}, severity = {1}, message = {2:?}",
            gltype,
            severity,
            CStr::from_ptr(message as *const c_char)
        );
    }
    if gltype == gl::DEBUG_TYPE_ERROR {
        panic!();
    }
}

fn main() {
    const WIDTH: f32 = 1600.0;
    const HEIGHT: f32 = 900.0;

    let el = event_loop::EventLoop::new();
    let wb = window::WindowBuilder::new()
        .with_title("Hello world!")
        .with_inner_size(dpi::LogicalSize::new(WIDTH as f64, HEIGHT as f64));

    let windowed_context = unsafe {
        ContextBuilder::new()
            .with_vsync(true)
            //.with_srgb(false)
            //.with_multisampling(4)
            .with_gl(GlRequest::Latest)
            .with_gl_profile(GlProfile::Core)
            .build_windowed(wb, &el)
            .unwrap()
            .make_current()
            .unwrap()
    };

    let window = windowed_context.window();
    window.set_cursor_visible(false);
    window.set_cursor_grab(true).unwrap();
    let (framebuffer_width, framebuffer_height): (u32, u32) = window
        .inner_size()
        .to_physical(window.hidpi_factor())
        .into();

    println!("{:?}", windowed_context.get_pixel_format());

    unsafe {
        gl::load_with(|s| windowed_context.get_proc_address(s));
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
    }

    println!("OpenGL info");
    unsafe {
        println!(
            " Vendor:   {:?}",
            CStr::from_ptr(gl::GetString(gl::VENDOR) as *const i8)
        );
        println!(
            " Renderer: {:?}",
            CStr::from_ptr(gl::GetString(gl::RENDERER) as *const i8)
        );
        println!(
            " Version:  {:?}",
            CStr::from_ptr(gl::GetString(gl::VERSION) as *const i8)
        );
    }

    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(Some(debug_callback), std::ptr::null());

        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        
        gl::Enable(gl::CULL_FACE);

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);
    }

    let (skybox_va, _skybox_vb) = create_skybox_buffers();
    let skybox_shader = Shader::new("../shaders/skybox.vert", "../shaders/skybox.frag").unwrap();

    const ENV_MAP_FACE_RESOLUTION: i32 = 1024;
    let skybox_texture = TextureCubeMap::new_from_hdr(
        "../resources/Factory_Catwalk/Factory_Catwalk_2k.hdr",
        ENV_MAP_FACE_RESOLUTION,
    )
    .unwrap();

    let irradiance_map = compute_irradiance_map(&skybox_texture);
    let prefiltered_env_map = compute_prefiltered_env_map(&skybox_texture, ENV_MAP_FACE_RESOLUTION);
    let lut_texture = compute_lut_texture(512);

    let sphere_shader =
        Shader::new("../shaders/sphere_pbr.vert", "../shaders/sphere_pbr.frag").unwrap();
    sphere_shader.set_uniform_3f("albedo", &vec3(0.5, 0.5, 0.5));
    sphere_shader.set_uniform_1f("ao", &1.0);
    sphere_shader.set_uniform_1i("irradiance_map", &0);
    sphere_shader.set_uniform_1i("prefiltered_map", &1);
    sphere_shader.set_uniform_1i("brdf_lut", &2);

    let light_positions = [
        Vector3::<f32> {
            x: -10.0,
            y: 10.0,
            z: 10.0,
        },
        Vector3::<f32> {
            x: 10.0,
            y: 10.0,
            z: 10.0,
        },
        Vector3::<f32> {
            x: -10.0,
            y: -10.0,
            z: 10.0,
        },
        Vector3::<f32> {
            x: 10.0,
            y: -10.0,
            z: 10.0,
        },
    ];

    let light_colors = [
        Vector3::<f32> {
            x: 150.0,
            y: 150.0,
            z: 150.0,
        },
        Vector3::<f32> {
            x: 150.0,
            y: 150.0,
            z: 150.0,
        },
        Vector3::<f32> {
            x: 150.0,
            y: 150.0,
            z: 150.0,
        },
        Vector3::<f32> {
            x: 150.0,
            y: 150.0,
            z: 150.0,
        },
    ];

    let (sphere_va, _sphere_vb, sphere_ib) = crate_sphere_buffers(1.0);

    let mut cam = Camera::new_default(WIDTH, HEIGHT);
    cam.position.z = 5.0;
    const CAM_SPEED: f32 = 0.00003;
    const FOV_SPEED: f32 = 1.05;
    const MOUSE_SPEED: f32 = 0.002;

    let mut window_focused = true;

    unsafe {
        gl::Viewport(0, 0, framebuffer_width as i32, framebuffer_height as i32);
    }

    let mut time = Instant::now();
    let mut delta_t = time.elapsed();

    let mut moving_up = false;
    let mut moving_down = false;
    let mut moving_right = false;
    let mut moving_left = false;

    use event::*;
    el.run(move |event, _, control_flow| {
        *control_flow = event_loop::ControlFlow::Poll;
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = event_loop::ControlFlow::Exit,
                WindowEvent::Resized(logical_size) => {
                    let dpi_factor = windowed_context.window().hidpi_factor();
                    windowed_context.resize(logical_size.to_physical(dpi_factor));
                }
                WindowEvent::Focused(f) => {
                    window_focused = *f;
                    if window_focused {
                        windowed_context.window().set_cursor_grab(true).ok();
                        windowed_context.window().set_cursor_visible(false);
                    } else {
                        windowed_context.window().set_cursor_grab(false).ok();
                        windowed_context.window().set_cursor_visible(true);
                    }
                }
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    ..
                } => {
                    windowed_context.window().set_cursor_grab(true).ok();
                    windowed_context.window().set_cursor_visible(false);
                    window_focused = true;
                }
                WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                    Some(VirtualKeyCode::Escape) => {
                        *control_flow = event_loop::ControlFlow::Exit;
                    }
                    Some(VirtualKeyCode::Up) => {
                        moving_up = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }
                    Some(VirtualKeyCode::Down) => {
                        moving_down = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }
                    Some(VirtualKeyCode::Right) => {
                        moving_right = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }
                    Some(VirtualKeyCode::Left) => {
                        moving_left = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }
                    Some(VirtualKeyCode::W) => {
                        cam.perspective.fovy /= FOV_SPEED;
                        if cam.perspective.fovy < Rad::from(Deg(15.0)) {
                            cam.perspective.fovy = Rad::from(Deg(15.0));
                        }
                    }
                    Some(VirtualKeyCode::S) => {
                        cam.perspective.fovy *= FOV_SPEED;
                        if cam.perspective.fovy > Rad::from(Deg(100.0)) {
                            cam.perspective.fovy = Rad::from(Deg(100.0));
                        }
                    }
                    Some(VirtualKeyCode::R) => {
                        let size = windowed_context.window().inner_size();
                        cam = Camera::new_default(size.width as f32, size.height as f32);
                        cam.position.z = 5.0;
                    }
                    _ => (),
                },
                WindowEvent::RedrawRequested => {
                    if moving_up {
                        cam.position += cam.direction() * delta_t.as_micros() as f32 * CAM_SPEED;
                    }
                    if moving_down {
                        cam.position -= cam.direction() * delta_t.as_micros() as f32 * CAM_SPEED;
                    }
                    if moving_right {
                        cam.position += cam.right() * delta_t.as_micros() as f32 * CAM_SPEED;
                    }
                    if moving_left {
                        cam.position -= cam.right() * delta_t.as_micros() as f32 * CAM_SPEED;
                    }

                    unsafe {
                        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    }
                                     let (view, projection) = cam.to_vp();
                    sphere_shader.bind();
                    sphere_shader.set_uniform_mat4f("projection", &projection);
                    sphere_shader.set_uniform_mat4f("view", &view);
                    let cam_pos = cam.position.to_homogeneous().truncate();
                    sphere_shader.set_uniform_3f("world_cam_posiiton", &cam_pos);
                    irradiance_map.set_slot(&0);
                    prefiltered_env_map.set_slot(&1);
                    lut_texture.set_slot(&2);

                    for i in 0..light_positions.len() {
                        sphere_shader.set_uniform_3f(
                            &format!("light_positions[{}]", i),
                            &light_positions[i],
                        );
                        sphere_shader
                            .set_uniform_3f(&format!("light_colors[{}]", i), &light_colors[i]);
                    }

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
                            draw_sphere(&sphere_ib, &sphere_va);
                        }
                    }

                    for i in 0..light_positions.len() {
                        let model = Matrix4::<f32>::from_translation(light_positions[i])
                            * Matrix4::<f32>::from_scale(0.5);

                        sphere_shader.set_uniform_mat4f("model", &model);
                        draw_sphere(&sphere_ib, &sphere_va);
                    }
                    let skybox_view = {
                        let mut v = view.clone();
                        v[3][0] = 0.0;
                        v[3][1] = 0.0;
                        v[3][2] = 0.0;
                        v[3][3] = 0.0;
                        v[0][3] = 0.0;
                        v[1][3] = 0.0;
                        v[2][3] = 0.0;
                        v
                    };
                    skybox_shader.bind();
                    skybox_shader.set_uniform_mat4f("view", &skybox_view);
                    skybox_shader.set_uniform_mat4f("projection", &projection);
                    skybox_shader.set_texture_slot("skybox", &0);
                    skybox_texture.set_slot(&0);

                    draw_skybox(&skybox_va);

                    windowed_context.swap_buffers().unwrap();

                    delta_t = time.elapsed();
                    time = Instant::now();
                    //println!("Time: {}ms", delta_t.as_micros() as f32 / 1000.0);
                }
                _ => (),
            },
            Event::DeviceEvent { ref event, .. } => match event {
                DeviceEvent::MouseMotion { delta: (x, y) } => {
                    cam.horizontal_angle -= Rad(*x as f32 * MOUSE_SPEED);
                    cam.vertical_angle -= Rad(*y as f32 * MOUSE_SPEED);
                }
                _ => (),
            },
            _ => (),
        }
        if window_focused {
            windowed_context.window().request_redraw();
        }
    });
}
