extern crate gl;
extern crate glutin;

pub mod pbr_spheres;
pub use pbr_spheres::*;

use std::collections::HashMap;
use std::time::Duration;

use glutin::event::*;
use glutin::event_loop::*;

pub struct TestApp<'a> {
    current_test: Option<Box<dyn TestScene + 'a>>,
    framebuffer_size: &'a (u32, u32),
    scenes_map: HashMap<VirtualKeyCode, (String, fn(&'a (u32, u32)) -> Box<dyn TestScene + 'a>)>,
}

impl<'a> TestApp<'a> {
    pub fn new(framebuffer_size: &'a (u32, u32)) -> Self {
        let mut res = Self {
            current_test: None,
            framebuffer_size,
            scenes_map: HashMap::new(),
        };
        res.reset();
        res
    }

    fn register<T: TestScene>(&mut self, name: &str, key: VirtualKeyCode) {
        self.scenes_map.insert(key, (name.to_string(), T::new));
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
                    Some(VirtualKeyCode::Escape) => match &mut self.current_test {
                        Some(_) => {
                            self.current_test = None;
                            self.reset();
                        }
                        None => *control_flow = ControlFlow::Exit,
                    },
                    Some(VirtualKeyCode::R) => {
                        self.reset();
                        self.print_map();
                    }
                    Some(key) => match self.scenes_map.get(&key) {
                        Some((_, fun)) => self.current_test = Some(fun(self.framebuffer_size)),
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

trait TestScene {
    fn new<'a>(framebuffer_size: &'a(u32, u32)) -> Box<dyn TestScene + 'a>
    where
        Self: Sized;
    fn reset(&mut self);
    fn handle_event(&mut self, event: &Event<()>, control_flow: &mut ControlFlow);
    fn update(&mut self, delta: Duration);
    fn render(&self);
}
