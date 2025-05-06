struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    time: f32,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) dir: vec3<f32>,
};


@vertex
fn vs_main(@location(0) in_pos: vec3<f32>) -> VertexOutput {
    var out: VertexOutput;

    var view_rot = mat4x4<f32>(
        camera.view_proj[0].xyzw,
        camera.view_proj[1].xyzw,
        camera.view_proj[2].xyzw,
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );

    out.pos = view_rot * vec4<f32>(in_pos, 1.0);
    out.dir = in_pos;
    return out;
}

// @group(0) @binding(1) var skybox_texture: texture_cube<f32>;
// @group(0) @binding(2) var skybox_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let color = textureSample(skybox_texture, skybox_sampler, normalize(in.dir));
    let color = vec4<f32>(135.0 / 255.0, 206.0/ 255.0, 235.0/ 255.0, 1.0); // Default sky color (sky blue)
    return color;
}