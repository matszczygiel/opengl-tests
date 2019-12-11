extern crate cgmath;

use crate::buffers::*;

use cgmath::*;

pub fn crate_sphere_buffers(radius: f32) -> (VertexArray, VertexBuffer, IndexBuffer) {
    const X_SEGMENTS: u32 = 64;
    const Y_SEGMENTS: u32 = 64;

    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    for y in 0..=Y_SEGMENTS {
        for x in 0..=X_SEGMENTS {
            let x_seg = x as f32 / X_SEGMENTS as f32;
            let y_seg = y as f32 / Y_SEGMENTS as f32;
            use std::f32::consts::PI;
            let x_pos = (x_seg * 2.0 * PI).cos() * (y_seg * PI).sin();
            let y_pos = (y_seg * PI).cos();
            let z_pos = (x_seg * 2.0 * PI).sin() * (y_seg * PI).sin();
            positions.push(Vector3 {
                x: radius * x_pos,
                y: radius * y_pos,
                z: radius * z_pos,
            });
            uvs.push(Vector2 { x: x_seg, y: y_seg });
            normals.push(Vector3 {
                x: x_pos,
                y: y_pos,
                z: z_pos,
            });
        }
    }
    let mut indices = Vec::<u32>::new();
    let mut odd_row = false;
    for y in 0..Y_SEGMENTS {
        if odd_row {
            for x in 0..=X_SEGMENTS {
                indices.push((y + 1) * (X_SEGMENTS + 1) + x);
                indices.push(y * (X_SEGMENTS + 1) + x);
            }
        } else {
            for x in (0..=X_SEGMENTS).rev() {
                indices.push(y * (X_SEGMENTS + 1) + x);
                indices.push((y + 1) * (X_SEGMENTS + 1) + x);
            }
        }
        odd_row = !odd_row;
    }

    let data = positions
        .into_iter()
        .zip(uvs.into_iter())
        .zip(normals.into_iter())
        .map(|((pos, uv), norm)| vec![pos.x, pos.y, pos.z, uv.x, uv.y, norm.x, norm.y, norm.z])
        .flatten()
        .collect::<Vec<f32>>();

    let va = VertexArray::new();
    va.bind();
    let vb = VertexBuffer::new_static(&data);
    vb.bind();
    const STRIDE: i32 = 3 + 2 + 3;
    va.set_vertex_attrib_array(0, 3, false, STRIDE, 0);
    va.set_vertex_attrib_array(1, 2, false, STRIDE, 3);
    va.set_vertex_attrib_array(2, 3, false, STRIDE, 5);
    let ib = IndexBuffer::new_static(&indices);

    (va, vb, ib)
}

pub fn draw_sphere(sphere_ib: &IndexBuffer, sphere_va: &VertexArray) {
    sphere_va.bind();
    sphere_ib.bind();
    unsafe {
        gl::DrawElements(
            gl::TRIANGLE_STRIP,
            sphere_ib.count() as i32,
            gl::UNSIGNED_INT,
            std::ptr::null(),
        );
    }
}

pub fn crate_cube_buffers() -> (VertexArray, VertexBuffer) {
    #[rustfmt::skip]
    const VERTICES: [f32; 8*6*6] = [
        // back face
        -1.0, -1.0, -1.0,  0.0, 0.0,  0.0,  0.0, -1.0, // bottom-left
         1.0,  1.0, -1.0,  1.0, 1.0,  0.0,  0.0, -1.0, // top-right
         1.0, -1.0, -1.0,  1.0, 0.0,  0.0,  0.0, -1.0, // bottom-right         
         1.0,  1.0, -1.0,  1.0, 1.0,  0.0,  0.0, -1.0, // top-right
        -1.0, -1.0, -1.0,  0.0, 0.0,  0.0,  0.0, -1.0, // bottom-left
        -1.0,  1.0, -1.0,  0.0, 1.0,  0.0,  0.0, -1.0, // top-left
        // front face
        -1.0, -1.0,  1.0,  0.0, 0.0,  0.0,  0.0,  1.0, // bottom-left
         1.0, -1.0,  1.0,  1.0, 0.0,  0.0,  0.0,  1.0, // bottom-right
         1.0,  1.0,  1.0,  1.0, 1.0,  0.0,  0.0,  1.0, // top-right
         1.0,  1.0,  1.0,  1.0, 1.0,  0.0,  0.0,  1.0, // top-right
        -1.0,  1.0,  1.0,  0.0, 1.0,  0.0,  0.0,  1.0, // top-left
        -1.0, -1.0,  1.0,  0.0, 0.0,  0.0,  0.0,  1.0, // bottom-left
        // left face
        -1.0,  1.0,  1.0,  1.0, 0.0, -1.0,  0.0,  0.0, // top-right
        -1.0,  1.0, -1.0,  1.0, 1.0, -1.0,  0.0,  0.0, // top-left
        -1.0, -1.0, -1.0,  0.0, 1.0, -1.0,  0.0,  0.0, // bottom-left
        -1.0, -1.0, -1.0,  0.0, 1.0, -1.0,  0.0,  0.0, // bottom-left
        -1.0, -1.0,  1.0,  0.0, 0.0, -1.0,  0.0,  0.0, // bottom-right
        -1.0,  1.0,  1.0,  1.0, 0.0, -1.0,  0.0,  0.0, // top-right
        // right face
         1.0,  1.0,  1.0,  1.0, 0.0,  1.0,  0.0,  0.0, // top-left
         1.0, -1.0, -1.0,  0.0, 1.0,  1.0,  0.0,  0.0, // bottom-right
         1.0,  1.0, -1.0,  1.0, 1.0,  1.0,  0.0,  0.0, // top-right         
         1.0, -1.0, -1.0,  0.0, 1.0,  1.0,  0.0,  0.0, // bottom-right
         1.0,  1.0,  1.0,  1.0, 0.0,  1.0,  0.0,  0.0, // top-left
         1.0, -1.0,  1.0,  0.0, 0.0,  1.0,  0.0,  0.0, // bottom-left     
        // bottom face
        -1.0, -1.0, -1.0,  0.0, 1.0,  0.0, -1.0,  0.0, // top-right
         1.0, -1.0, -1.0,  1.0, 1.0,  0.0, -1.0,  0.0, // top-left
         1.0, -1.0,  1.0,  1.0, 0.0,  0.0, -1.0,  0.0, // bottom-left
         1.0, -1.0,  1.0,  1.0, 0.0,  0.0, -1.0,  0.0, // bottom-left
        -1.0, -1.0,  1.0,  0.0, 0.0,  0.0, -1.0,  0.0, // bottom-right
        -1.0, -1.0, -1.0,  0.0, 1.0,  0.0, -1.0,  0.0, // top-right
        // top face
        -1.0,  1.0, -1.0,  0.0, 1.0,  0.0,  1.0,  0.0, // top-left
         1.0,  1.0,  1.0,  1.0, 0.0,  0.0,  1.0,  0.0, // bottom-right
         1.0,  1.0, -1.0,  1.0, 1.0,  0.0,  1.0,  0.0, // top-right     
         1.0,  1.0,  1.0,  1.0, 0.0,  0.0,  1.0,  0.0, // bottom-right
        -1.0,  1.0, -1.0,  0.0, 1.0,  0.0,  1.0,  0.0, // top-left
        -1.0,  1.0,  1.0,  0.0, 0.0,  0.0,  1.0,  0.0 // bottom-left        
    ];
    let va = VertexArray::new();
    va.bind();
    let vb = VertexBuffer::new_static(&VERTICES);
    vb.bind();
    const STRIDE: i32 = 3 + 2 + 3;
    va.set_vertex_attrib_array(0, 3, false, STRIDE, 0);
    va.set_vertex_attrib_array(1, 2, false, STRIDE, 3);
    va.set_vertex_attrib_array(2, 3, false, STRIDE, 5);
    (va, vb)
}

pub fn draw_cube(va: &VertexArray) {
    va.bind();
    unsafe {
        gl::DrawArrays(gl::TRIANGLES, 0, 36);
    }
}

pub fn create_skybox_buffers() -> (VertexArray, VertexBuffer) {
    #[rustfmt::skip]
    const VERTICES: [f32; 6*6*3] = [
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

    let va = VertexArray::new();
    va.bind();
    let vb = VertexBuffer::new_static(&VERTICES);
    vb.bind();
    const STRIDE: i32 = 3;
    va.set_vertex_attrib_array(0, 3, false, STRIDE, 0);
    (va, vb)
}

pub fn draw_skybox(va: &VertexArray) {
    va.bind();
    unsafe {
        gl::DepthFunc(gl::LEQUAL);
        gl::DrawArrays(gl::TRIANGLES, 0, 36);
        gl::DepthFunc(gl::LESS);
    }
}

pub fn create_quad_buffers() -> (VertexArray, VertexBuffer) {
    #[rustfmt::skip]
    const VERTICES: [f32; 4* 5] = [
        -1.0,  1.0, 0.0,    0.0, 1.0,
        -1.0, -1.0, 0.0,    0.0, 0.0,
         1.0,  1.0, 0.0,    1.0, 1.0,
         1.0, -1.0, 0.0,    1.0, 0.0,
    ];
    let va = VertexArray::new();
    va.bind();
    let vb = VertexBuffer::new_static(&VERTICES);
    vb.bind();
    const STRIDE: i32 = 5;
    va.set_vertex_attrib_array(0, 3, false, STRIDE, 0);
    va.set_vertex_attrib_array(1, 2, false, STRIDE, 3);
    (va, vb)
}

pub fn draw_quad(va: &VertexArray) {
    va.bind();
    unsafe {
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
    }
}
