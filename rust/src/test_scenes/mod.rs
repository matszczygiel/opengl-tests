extern crate gl;
extern crate glutin;

pub mod pbr_spheres;
pub use pbr_spheres::*;
pub mod pbr_textured_spheres;
pub use pbr_textured_spheres::*;

use std::collections::HashMap;
use std::time::Duration;

use glutin::event::*;
use glutin::event_loop::*;

pub struct TestApp {
    current_test: Option<Box<dyn TestScene>>,
    scenes_map: HashMap<VirtualKeyCode, (String, fn((u32, u32)) -> Box<dyn TestScene>)>,
    framebuffer_size: (u32, u32),
}

impl TestApp {
    pub fn new(framebuffer_size: (u32, u32)) -> Self {
        let mut res = Self {
            current_test: None,
            scenes_map: HashMap::new(),
            framebuffer_size: (0, 0),
        };
        res.reset();
        res.set_framebuffer_size(framebuffer_size);
        res
    }

    pub fn register<T: TestScene>(&mut self, name: &str, key: VirtualKeyCode) {
        self.scenes_map.insert(key, (name.to_string(), T::new));
    }

    pub fn set_framebuffer_size(&mut self, size: (u32, u32)) {
        self.framebuffer_size = size;
        match &mut self.current_test {
            Some(test) => test.set_framebuffer_size(size),
            None => unsafe {
                gl::Viewport(
                    0,
                    0,
                    self.framebuffer_size.0 as i32,
                    self.framebuffer_size.1 as i32,
                );
            },
        }
    }

    fn print_map(&self) {
        println!("Available test scenes:");
        for (key, (name, _)) in &self.scenes_map {
            println!("Key: {:?}, scene: {}", key, name);
        }
    }

    fn reset(&mut self) {
        match &mut self.current_test {
            Some(test) => test.reset(),
            None => unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            },
        }
    }

    pub fn handle_event(&mut self, event: &Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                    Some(VirtualKeyCode::Back) => match &mut self.current_test {
                        None => (),
                        Some(_) => {
                            self.current_test = None;
                            self.reset();
                        }
                    },
                    Some(VirtualKeyCode::Escape) => *control_flow = ControlFlow::Exit,
                    Some(VirtualKeyCode::R) => {
                        self.reset();
                        if self.current_test.is_none() {
                            self.print_map();
                        }
                    }
                    Some(key) => match self.scenes_map.get(&key) {
                        Some((_, fun)) => match self.current_test {
                            Some(_) => (),
                            None => self.current_test = Some(fun(self.framebuffer_size)),
                        },
                        None => (),
                    },
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }

        match &mut self.current_test {
            Some(test) => test.handle_event(event, control_flow),
            _ => (),
        }
    }

    pub fn update(&mut self, delta_time: Duration) {
        match &mut self.current_test {
            Some(test) => test.update(delta_time),
            _ => (),
        }
    }

    pub fn render(&self) {
        match &self.current_test {
            Some(test) => test.render(),
            None => unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
            },
        }
    }
}

pub trait TestScene {
    fn new(framebuffer_size: (u32, u32)) -> Box<dyn TestScene>
    where
        Self: Sized;
    fn reset(&mut self);
    fn handle_event(&mut self, event: &Event<()>, control_flow: &mut ControlFlow);
    fn update(&mut self, delta: Duration);
    fn render(&self);
    fn set_framebuffer_size(&mut self, size: (u32, u32));
}
