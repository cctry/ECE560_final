// ===================
// UNIFORMS
// ===================

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    time: f32,
}
@group(0) @binding(0)
var<uniform> camera: Camera;


// ===================
// STRUCTS
// ===================

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
}

// ===================
// VERTEX SHADER
// ===================

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.view_proj * vec4<f32>(position, 1.0);
    output.uv = (position.xz + vec2<f32>(256.0, 256.0)) / 512.0;
    output.world_pos = position;
    return output;
}

// ===================
// FRAGMENT SHADER
// ===================

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // === Dynamic UV offset ===
    let flow_speed = vec2<f32>(0.1, 0.05);
    let uv = input.uv + camera.time * flow_speed;

    // === Procedural wave pattern ===
    let wave1 = sin(uv.x * 10.0 + camera.time) * 0.02;
    let wave2 = sin(uv.y * 12.0 + camera.time * 1.2) * 0.015;
    let wave3 = sin((uv.x + uv.y) * 14.0 + camera.time * 0.8) * 0.01;
    let wave = wave1 + wave2 + wave3;

    // === Base water color ===
    let base_color = vec3<f32>(0.0, 0.5, 0.8);
    let wave_color = base_color + vec3<f32>(wave);

    // Dynamic normal for Fresnel
    let dx = cos(uv.x * 8.0 + camera.time) * 0.2;
    let dz = sin(uv.y * 10.0 + camera.time) * 0.2;
    let normal = normalize(vec3<f32>(dx, 1.0, dz));

    // Fresnel term
    let view_dir = normalize(camera.view_pos.xyz - input.world_pos);
    let fresnel = pow(1.0 - dot(normal, view_dir), 4.0);

    let final_color = mix(wave_color, vec3<f32>(1.0), fresnel * 0.6);
    let alpha = mix(0.4, 0.7, fresnel);

    return vec4<f32>(final_color, alpha);
}
