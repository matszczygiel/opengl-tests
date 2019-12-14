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
mod test_scenes;
mod textures;
mod utils;

use buffers::*;
use camera::*;
use shaders::*;
use test_scenes::*;
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
            .with_multisampling(4)
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

    let mut framebuffer_size: (u32, u32) = window
        .inner_size()
        .to_physical(window.hidpi_factor())
        .into();

    println!("{:?}", windowed_context.get_pixel_format());

    unsafe {
        gl::load_with(|s| windowed_context.get_proc_address(s));
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(Some(debug_callback), std::ptr::null());
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

    let test_app = TestApp::new(&framebuffer_size);

    let mut cam = Camera::new_default(WIDTH, HEIGHT);
    cam.position.z = 5.0;

    let mut window_focused = true;

    unsafe {
        gl::Viewport(0, 0, framebuffer_size.0 as i32, framebuffer_size.1 as i32);
    }

    let mut time = Instant::now();
    let mut delta_t = time.elapsed();

    use event::*;
    el.run(move |event, _, control_flow| {
        *control_flow = event_loop::ControlFlow::Poll;
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = event_loop::ControlFlow::Exit,
                WindowEvent::Resized(logical_size) => {
                    let dpi_factor = windowed_context.window().hidpi_factor();
                    let size = logical_size.to_physical(dpi_factor);
                    windowed_context.resize(size);
                    framebuffer_size = size.into();
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
                WindowEvent::RedrawRequested => {
                    test_app.update(delta_t);
                    test_app.render();

                    windowed_context.swap_buffers().unwrap();

                    delta_t = time.elapsed();
                    time = Instant::now();
                    //println!("Time: {}ms", delta_t.as_micros() as f32 / 1000.0);
                }
                _ => (),
            },
            _ => (),
        }

        test_app.handle_event(&event, &mut control_flow);
        if window_focused {
            windowed_context.window().request_redraw();
        }
    });
}
