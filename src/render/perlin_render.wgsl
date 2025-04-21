// WGSL Vertex/Fragment Shader for Display

// Input texture (height map)
@group(0) @binding(0)
var height_texture: texture_2d<f32>;
@group(0) @binding(1)
var height_sampler: sampler;

// Vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Vertex shader (full-screen quad)
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 0.0)
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.uv = uvs[vertex_index];
    return output;
}

fn terrain_colormap(x: f32) -> vec3<f32> {
    let x_clamped = clamp(x, 0.0, 1.0);

    if (x_clamped <= 0.15) {
        let t = x_clamped / 0.15;
        return mix(vec3<f32>(0.2, 0.2, 0.6), vec3<f32>(0.0, 0.6, 1.0), t);
    } else if (x_clamped <= 0.25) {
        let t = (x_clamped - 0.15) / (0.25 - 0.15);
        return mix(vec3<f32>(0.0, 0.6, 1.0), vec3<f32>(0.0, 0.8, 0.4), t);
    } else if (x_clamped <= 0.50) {
        let t = (x_clamped - 0.25) / (0.50 - 0.25);
        return mix(vec3<f32>(0.0, 0.8, 0.4), vec3<f32>(1.0, 1.0, 0.6), t);
    } else if (x_clamped <= 0.75) {
        let t = (x_clamped - 0.50) / (0.75 - 0.50);
        return mix(vec3<f32>(1.0, 1.0, 0.6), vec3<f32>(0.5, 0.36, 0.33), t);
    } else {
        let t = (x_clamped - 0.75) / (1.00 - 0.75);
        return mix(vec3<f32>(0.5, 0.36, 0.33), vec3<f32>(1.0, 1.0, 1.0), t);
    }
}

// Fragment shader
@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    // Sample height value
    let height = textureSample(height_texture, height_sampler, uv).r;
    
    // Convert to color (grayscale)
    let color = terrain_colormap(height);
    
    return vec4<f32>(color, 1.0);
}