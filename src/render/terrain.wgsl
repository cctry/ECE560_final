struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    time: f32,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) height: f32,
}


@vertex
fn vs_main(@location(0) position: vec3<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.view_proj * vec4<f32>(position, 1.0);
    
    // Pass the y component (height) to the fragment shader
    output.height = position.y;
    
    return output;
}

fn terrain_colormap(x: f32) -> vec3<f32> {
    if (x <= 0.15) {
        let t = x / 0.15;
        return mix(vec3<f32>(0.2, 0.2, 0.6), vec3<f32>(0.0, 0.6, 1.0), t);
    } else if (x <= 0.375) {
        let t = (x - 0.15) / (0.375 - 0.15);
        return mix(vec3<f32>(0.0, 0.6, 1.0), vec3<f32>(0.0, 0.8, 0.4), t);
    } else if (x <= 0.85) {
        let t = (x - 0.375) / (0.85 - 0.375);
        return mix(vec3<f32>(0.0, 0.8, 0.4), vec3<f32>(1.0, 1.0, 0.6), t);
    } else if (x <= 0.95) {
        let t = (x - 0.85) / (0.95 - 0.85);
        return mix(vec3<f32>(1.0, 1.0, 0.6), vec3<f32>(0.5, 0.36, 0.33), t);
    } else {
        let t = (x - 0.95) / (1.00 - 0.95);
        return mix(vec3<f32>(0.5, 0.36, 0.33), vec3<f32>(1.0, 1.0, 1.0), t);
    }
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let h = (input.height + 16.0) / 32.0;
    return vec4<f32>(terrain_colormap(h), 1.0);
}
