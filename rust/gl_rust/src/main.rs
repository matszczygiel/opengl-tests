extern crate gl;
extern crate glutin;

fn main() {
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Hello world!")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));

    let windowed_context = unsafe {
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(wb, &el)
            .unwrap()
            .make_current()
            .unwrap()
    };

    unsafe {
        gl::load_with(|s| windowed_context.get_proc_address(s));
        gl::ClearColor(0.0, 0.0, 0.4, 0.0);
    }

    el.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::LoopDestroyed => return,
            glutin::event::Event::WindowEvent { ref event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit
                }
                glutin::event::WindowEvent::Resized(logical_size) => {
                    let dpi_factor = windowed_context.window().hidpi_factor();
                    windowed_context.resize(logical_size.to_physical(dpi_factor));
                }
                glutin::event::WindowEvent::RedrawRequested => {
                    unsafe {
                        gl::Clear(gl::COLOR_BUFFER_BIT);
                    }
                    windowed_context.swap_buffers().unwrap();
                }
                _ => (),
            },
            glutin::event::Event::DeviceEvent { ref event, .. } => match event {
                glutin::event::DeviceEvent::Key(key) => match key.virtual_keycode {
                    Some(glutin::event::VirtualKeyCode::Escape) => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
    });
}
