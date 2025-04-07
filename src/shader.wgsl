// Common structures
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0); // 2.
    return out;
}


@fragment
fn fs_main(
    @location(0) color: vec3<f32>,
) -> @location(0) vec4<f32> {
    return vec4<f32>(color, 1.0);
}

// Crosshair vertex shader
struct CrosshairVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_crosshair(@builtin(vertex_index) vertex_index: u32) -> CrosshairVertexOutput {
    var pos = array<vec2<f32>, 12>(
        // Vertical line
        vec2<f32>(-0.002, 0.02),  // Top
        vec2<f32>(0.002, 0.02),
        vec2<f32>(0.002, -0.02),  // Bottom
        vec2<f32>(-0.002, 0.02),  // Top
        vec2<f32>(0.002, -0.02),
        vec2<f32>(-0.002, -0.02), // Bottom
        
        // Horizontal line
        vec2<f32>(0.02, -0.002),  // Right
        vec2<f32>(0.02, 0.002),
        vec2<f32>(-0.02, 0.002),  // Left
        vec2<f32>(0.02, -0.002),  // Right
        vec2<f32>(-0.02, 0.002),
        vec2<f32>(-0.02, -0.002)  // Left
    );
    
    var out: CrosshairVertexOutput;
    out.clip_position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
    out.color = vec3<f32>(1.0, 1.0, 1.0); // White color
    return out;
}

@fragment
fn fs_crosshair(@location(0) color: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(color, 1.0);
}
