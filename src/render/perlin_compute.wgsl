struct Parameters {
    persistence: f32,
    octaves: u32,
    lacunarity: f32,
    scale: u32,
    p_table: array<vec4<u32>, 128>,
}

// Define bindings for the buffers
@group(0) @binding(0) var<uniform> params: Parameters;

const GRADIENTS: array<vec2<f32>, 8> = array<vec2<f32>, 8>(
    vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 0.0), vec2<f32>(-1.0, 0.0), vec2<f32>(0.0, 1.0), vec2<f32>(0.0, -1.0)
);

fn access_p(x_index: u32, y_index: u32) -> u32 {
    let x_vec_idx = x_index / 4u;
    let x_comp_idx = x_index % 4u;
    let perm_value = params.p_table[x_vec_idx][x_comp_idx];
    
    let combined = perm_value + y_index;
    let combined_vec_idx = combined / 4u;
    let combined_comp_idx = combined % 4u;
    
    return params.p_table[combined_vec_idx][combined_comp_idx];
}

fn fade(t: f32) -> f32 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    return a + t * (b - a);
}

// Gradient function
fn gradient(h: u32, x: f32, y: f32) -> f32 {
    let g = GRADIENTS[h & 7u];
    return g.x * x + g.y * y;
}

fn perlin_noise_2d(x: f32, y: f32) -> f32 {
    // Grid cell coordinates
    let xi = u32(floor(x));
    let yi = u32(floor(y));

    // Fractional coordinates
    let xf = x - f32(xi);
    let yf = y - f32(yi);

    // Fade curves
    let u = fade(xf);
    let v = fade(yf);

    // Hash coordinates of the 4 corners
    let ii = xi & 255u;
    let ji = yi & 255u;
    
    let h00 = access_p(ii, ji);
    let h10 = access_p(ii + 1u, ji);
    let h01 = access_p(ii, ji + 1u);
    let h11 = access_p(ii + 1u, ji + 1u);
    
    // Gradients
    let g00 = gradient(h00, xf, yf);
    let g10 = gradient(h10, xf - 1.0, yf);
    let g01 = gradient(h01, xf, yf - 1.0);
    let g11 = gradient(h11, xf - 1.0, yf - 1.0);

    // Interpolate
    let x1 = lerp(g00, g10, u);
    let x2 = lerp(g01, g11, u);
    return lerp(x1, x2, v);
}


fn fBm_2d(x: f32, y: f32) -> f32 {
    var total: f32 = 0.0;
    var amplitude: f32 = 1.0;
    var frequency: f32 = 1.0;
    for (var i = 0u; i < params.octaves; i = i + 1) {
        total = total + amplitude * perlin_noise_2d(x * frequency, y * frequency);
        amplitude = amplitude * params.persistence;
        frequency = frequency * params.lacunarity;
    }
    return total;
}

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

// Fragment shader
@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    // Scale coordinates
    let scaled_pos = uv * f32(params.scale);

    // Generate fBm noise
    let noise_value = fBm_2d(scaled_pos.x, scaled_pos.y);

    // Output noise value (normalized to [0, 1])
    return vec4<f32>((noise_value + 1.0) * 0.5, 0.0, 0.0, 1.0);
}