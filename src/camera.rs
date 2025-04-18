use cgmath::*;
use instant::Duration;
use std::f32::consts::FRAC_PI_2;
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::keyboard::KeyCode;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(aspect: f32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: aspect,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.amount_forward = amount;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.amount_backward = amount;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.amount_left = amount;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.amount_right = amount;
                true
            }
            KeyCode::Space => {
                self.amount_up = amount;
                true
            }
            KeyCode::ShiftLeft => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();
    
        // Compute forward direction based on yaw and pitch for cgmath (RH)
        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
    
        // Forward direction - matching Camera::calc_matrix
        let forward = Vector3::new(
            pitch_cos * yaw_cos,  // X component
            pitch_sin,            // Y component (up/down)
            pitch_cos * yaw_sin   // Z component
        )
        .normalize();
    
        // Right vector - correctly compute for strafing
        // Use a cross product with the world up vector (0,1,0)
        // In a right-handed coordinate system, right = forward × up
        let right = Vector3::new(
            -yaw_sin,             // X component 
            0.0,                  // Y component (no vertical movement for strafing)
            -yaw_cos              // Z component
        )
        .normalize();
    
        // Up vector perpendicular to forward and right
        let up = right.cross(forward).normalize();
    
        // Combine movements
        let mut direction = Vector3::zero();
        
        // Apply forward/backward movement
        if self.amount_forward > 0.0 || self.amount_backward > 0.0 {
            direction += forward * (self.amount_forward - self.amount_backward);
        }
        
        // Apply left/right movement (strafe)
        if self.amount_right > 0.0 || self.amount_left > 0.0 {
            direction += right * (self.amount_right - self.amount_left);
        }
        
        // Apply up/down movement
        if self.amount_up > 0.0 || self.amount_down > 0.0 {
            direction += up * (self.amount_up - self.amount_down);
        }
        
        // Normalize direction if there's any movement
        if direction.magnitude2() > 0.0 {
            direction = direction.normalize();
        }
    
        // Apply combined directional movement
        camera.position += direction * self.speed * dt;
    
        // Scroll zoom (optional)
        camera.position += forward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;
    
        // Rotate camera based on mouse input
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;
    
        // Clamp pitch angle to avoid flipping vertically
        camera.pitch.0 = camera.pitch.0.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);
    
        // Reset mouse deltas
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }
    
}
