extern crate cgmath;

use cgmath::*;

pub struct Camera {
    pub perspective: PerspectiveFov<f32>,
    pub position: Point3<f32>,
    pub horizontal_angle: Rad<f32>,
    pub vertical_angle: Rad<f32>,
}

impl Camera {
    pub fn new_default(width: f32, height: f32) -> Self {
        Self {
            perspective: PerspectiveFov {
                fovy: Rad::from(Deg(45.0)),
                aspect: width / height,
                near: 0.1,
                far: 100.0,
            },
            position: Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.5,
            },
            horizontal_angle: Rad(3.14),
            vertical_angle: Rad(0.0),
        }
    }

    pub fn direction(&self) -> Vector3<f32> {
        Vector3 {
            x: self.vertical_angle.cos() * self.horizontal_angle.sin(),
            y: self.vertical_angle.sin(),
            z: self.vertical_angle.cos() * self.horizontal_angle.sin(),
        }
    }

    pub fn right(&self) -> Vector3<f32> {
        Vector3 {
            x: (self.horizontal_angle - Rad(3.14 / 2.0)).sin(),
            y: 0.0,
            z: (self.horizontal_angle - Rad(3.14 / 2.0)).cos(),
        }
    }

    pub fn to_VP(&self) -> (Matrix4<f32>, Matrix4<f32>) {
        let direction = self.direction();
        let right = self.right();
        let up = right.cross(direction);

        let view = Matrix4::look_at_dir(self.position, direction, up);

        let projection = {
            let PerspectiveFov {
                fovy,
                aspect,
                near,
                far,
            } = self.perspective;
            perspective(fovy, aspect, near, far)
        };

        (view, projection)
    }
}
