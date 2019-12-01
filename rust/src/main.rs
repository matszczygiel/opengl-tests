extern crate cgmath;
extern crate gl;
extern crate glutin;

mod buffers;
mod camera;
mod shaders;
mod textures;

use buffers::*;
use camera::*;
use shaders::*;
use textures::*;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::time::{Duration, Instant};

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
}

fn main() {
    const WIDTH: f32 = 1600.0;
    const HEIGTH: f32 = 900.0;

    let el = event_loop::EventLoop::new();
    let wb = window::WindowBuilder::new()
        .with_title("Hello world!")
        .with_inner_size(dpi::LogicalSize::new(WIDTH as f64, HEIGTH as f64));

    let windowed_context = unsafe {
        ContextBuilder::new()
            .with_vsync(true)
            .with_hardware_acceleration(Some(true))
            .build_windowed(wb, &el)
            .unwrap()
            .make_current()
            .unwrap()
    };

    let window = windowed_context.window();
    //    window.set_cursor_visible(false);
    window.set_cursor_grab(true);

    unsafe {
        gl::load_with(|s| windowed_context.get_proc_address(s));
        gl::ClearColor(0.0, 0.0, 0.4, 0.0);
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
    }

    #[rustfmt::skip]
    let skybox_vertices: [f32; 6*6*3] = [
        -1.0,  1.0,  -1.0,
        -1.0, -1.0,  -1.0,
         1.0, -1.0,  -1.0,
         1.0, -1.0,  -1.0,
         1.0,  1.0,  -1.0,
        -1.0,  1.0,  -1.0,

        -1.0, -1.0,  1.0,
        -1.0, -1.0, -1.0,
        -1.0,  1.0, -1.0,
        -1.0,  1.0, -1.0,
        -1.0,  1.0,  1.0,
        -1.0, -1.0,  1.0,

         1.0, -1.0, -1.0,
         1.0, -1.0,  1.0,
         1.0,  1.0,  1.0,
         1.0,  1.0,  1.0,
         1.0,  1.0, -1.0,
         1.0, -1.0, -1.0,

        -1.0, -1.0,  1.0,
        -1.0,  1.0,  1.0,
         1.0,  1.0,  1.0,
         1.0,  1.0,  1.0,
         1.0, -1.0,  1.0,
        -1.0, -1.0,  1.0,

        -1.0,  1.0, -1.0,
         1.0,  1.0, -1.0,
         1.0,  1.0,  1.0,
         1.0,  1.0,  1.0,
        -1.0,  1.0,  1.0,
        -1.0,  1.0, -1.0,

        -1.0, -1.0, -1.0,
        -1.0, -1.0,  1.0,
         1.0, -1.0, -1.0,
         1.0, -1.0, -1.0,
        -1.0, -1.0,  1.0,
         1.0, -1.0,  1.0];

    let skybox_va = VertexArray::new();
    let skybox_vb = VertexBuffer::new_static(&skybox_vertices);
    let skybox_shader = Shader::new("../shaders/skybox.vert", "../shaders/skybox.frag").unwrap();
    let skybox_texture = TextureCubeMap::new_from_images(
        "../resources/cubemap/px.png",
        "../resources/cubemap/nx.png",
        "../resources/cubemap/py.png",
        "../resources/cubemap/ny.png",
        "../resources/cubemap/pz.png",
        "../resources/cubemap/nz.png",
    )
    .unwrap();

    let mut cam = Camera::new_default(WIDTH, HEIGTH);
    const CAM_SPEED: f32 = 3.0;
    const FOV_SPEED: f32 = 1.05;
    const MOUSE_SPEED: f32 = 0.002;

    let mut time = Instant::now();
    let mut delta_t = time.elapsed();
    el.run(move |event, _, control_flow| {
        match event {
            event::Event::LoopDestroyed => return,
            event::Event::WindowEvent { ref event, .. } => match event {
                event::WindowEvent::CloseRequested => *control_flow = event_loop::ControlFlow::Exit,
                event::WindowEvent::Resized(logical_size) => {
                    let dpi_factor = windowed_context.window().hidpi_factor();
                    windowed_context.resize(logical_size.to_physical(dpi_factor));
                }
                _ => (),
            },
            event::Event::DeviceEvent { ref event, .. } => match event {
                event::DeviceEvent::Key(key) => match key.virtual_keycode {
                    Some(event::VirtualKeyCode::Escape) => {
                        *control_flow = event_loop::ControlFlow::Exit;
                    }
                    Some(event::VirtualKeyCode::Up) => {
                        cam.position += cam.direction() * delta_t.as_micros() as f32 * CAM_SPEED;
                    }
                    Some(event::VirtualKeyCode::Down) => {
                        cam.position -= cam.direction() * delta_t.as_micros() as f32 * CAM_SPEED;
                    }
                    Some(event::VirtualKeyCode::Right) => {
                        cam.position += cam.right() * delta_t.as_micros() as f32 * CAM_SPEED;
                    }
                    Some(event::VirtualKeyCode::Left) => {
                        cam.position -= cam.right() * delta_t.as_micros() as f32 * CAM_SPEED;
                    }
                    Some(event::VirtualKeyCode::W) => {
                        cam.perspective.fovy /= FOV_SPEED;
                        if cam.perspective.fovy < Rad::from(Deg(15.0)) {
                            cam.perspective.fovy = Rad::from(Deg(15.0));
                        }
                    }
                    Some(event::VirtualKeyCode::S) => {
                        cam.perspective.fovy *= FOV_SPEED;
                        if cam.perspective.fovy > Rad::from(Deg(100.0)) {
                            cam.perspective.fovy = Rad::from(Deg(100.0));
                        }
                    }
                    _ => (),
                },
                event::DeviceEvent::MouseMotion { delta: (x, y) } => {
                    cam.horizontal_angle += Rad(*x as f32 * MOUSE_SPEED);
                    cam.vertical_angle += Rad(*y as f32 * MOUSE_SPEED);
                }
                _ => (),
            },
            _ => (),
        }
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        let (view, projection) = cam.to_VP();
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
        skybox_va.bind();
        skybox_shader.set_uniform_mat4f("view", &skybox_view);
        skybox_shader.set_uniform_mat4f("projection", &projection);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }
        skybox_texture.bind();
        skybox_shader.set_uniform_1i("skybox", &0);
        unsafe {
            gl::EnableVertexAttribArray(0);
            skybox_vb.bind();
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, 0 as *const std::ffi::c_void);
            gl::DepthFunc(gl::LEQUAL);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::DepthFunc(gl::LESS);
            gl::DisableVertexAttribArray(0);
        }

        windowed_context.swap_buffers().unwrap();

        delta_t = time.elapsed();
        time = Instant::now();
        println!("Time: {}ms", delta_t.as_micros());
    });

}
