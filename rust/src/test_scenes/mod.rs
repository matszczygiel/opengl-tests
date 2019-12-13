use std::time::Duration;

pub struct TestApp {
    current_test: Option<Box<dyn TestScene>>,
}

impl TestApp {
    pub fn new() -> Self {
        Self { current_test: None }
    }

    pub fn update(delta_time: Duration) {

    }

    pub fn render() {
        
    }
}

trait TestScene {
    fn update(delta: Duration);
    fn render();
}
