use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use winit::keyboard::KeyCode;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const YAW: Deg<f32> = Deg(-90.0); // 0 --> positive x-axis
const PITCH: Deg<f32> = Deg(0.0); // 90 --> positive y-axis

const SPEED: f32 = 8.0;
const SENSITIVITY: f32 = 1.0;
const ZOOM: Deg<f32> = Deg(45.0);
const ZNEAR: f32 = 0.1;
const ZFAR: f32 = 100000.0;
const WORLD_UP: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    pub view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

pub struct Camera {
    // state
    eye: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    // projection
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
    // movement
    speed: f32,
    sensitivity: f32,
    movement: Vector4<f32>,
    rotation: Vector2<f32>,
    // render
    buffer: wgpu::Buffer,
    uniform: CameraUniform,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Camera {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        Self {
            // state
            eye: (0.0, 10.0, 0.0).into(),
            yaw: YAW.into(),
            pitch: PITCH.into(),
            // projection
            aspect: width as f32 / height as f32,
            fovy: ZOOM.into(),
            znear: ZNEAR,
            zfar: ZFAR,
            // movement
            speed: SPEED,
            sensitivity: SENSITIVITY,
            movement: Vector4::zero(),
            rotation: Vector2::zero(),
            // render
            buffer: buffer,
            uniform: CameraUniform::new(),
            bind_group: bind_group,
            bind_group_layout: bind_group_layout,
        }
    }

    fn front_right_up(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let front = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();
        let right = front.cross(WORLD_UP).normalize();
        let up = right.cross(front).normalize();
        (front, right, up)
    }

    fn update_uniform(&mut self, front: Vector3<f32>, up: Vector3<f32>) {
        let center = self.eye + front;
        let view = Matrix4::look_at_rh(self.eye, center, up);
        let proj = perspective(self.fovy, self.aspect, self.znear, self.zfar);

        self.uniform.view_position = self.eye.to_homogeneous().into();
        self.uniform.view_proj = (OPENGL_TO_WGPU_MATRIX * proj * view).into();        
    }

    // called when the surface is resized
    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    // called for input events
    pub fn process_key(&mut self, key: &KeyCode) -> bool {
        match key {
            KeyCode::KeyW => {
                self.movement.x += 1.0;
                true
            }
            KeyCode::KeyS => {
                self.movement.y += 1.0;
                true
            }
            KeyCode::KeyA => {
                self.movement.z += 1.0;
                true
            }
            KeyCode::KeyD => {
                self.movement.w += 1.0;
                true
            }
            _ => false,
        }
    }

    // called before window event. This is device event
    pub fn process_mouse(&mut self, delta: Vector2<f32>) {
        self.rotation = delta;
    }

    // update everything before rendering
    pub fn update(&mut self, dt: &Duration, queue: &wgpu::Queue) {
        let dt = dt.as_secs_f32();
        self.yaw += Rad(self.rotation.x) * self.sensitivity * dt;
        self.pitch -= Rad(self.rotation.y) * self.sensitivity * dt;
        self.pitch = Rad(self.pitch.0.max(-FRAC_PI_2).min(FRAC_PI_2));
        let velocity = self.speed * dt;
        let (front, right, up) = self.front_right_up();
        self.eye += front * velocity * self.movement.x
            - front * velocity * self.movement.y
            - right * velocity * self.movement.z
            + right * velocity * self.movement.w;
        self.movement = Vector4::zero();
        self.rotation = Vector2::zero();
        self.update_uniform(front, up);
        // upload the uniform
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}

