# Procedural Terrain Generation with WebGPU

## Abstract

This project explores procedural terrain generation using WebGPU and modern graphics techniques, implemented as a web application using Rust and WebAssembly. The implementation combines Fractal Brownian Motion (FBM) with Perlin noise to generate terrain heightmaps, which are rendered using triangle strip-based mesh tessellation. The project features an interactive 3D environment with camera controls, skybox rendering, and dynamic terrain visualization. A height-based terrain colormap provides visual distinction between different elevation levels, demonstrating both procedural generation techniques and modern real-time graphics programming concepts.

## Introduction

Real-time terrain generation and rendering is a fundamental challenge in computer graphics, with applications ranging from video games to scientific visualization. This project implements a complete terrain generation pipeline using Rust, WebAssembly, and WebGPU, focusing on four key aspects:

1. Procedural terrain generation using advanced noise functions
2. Efficient mesh generation and GPU memory management
3. Real-time 3D rendering with multiple render passes
4. Interactive camera system with user controls

The implementation uses WebGPU for rendering and is structured into several key components including terrain generation, mesh creation, and visualization through custom shaders. The system can generate unique terrains through parameterized noise functions that can be adjusted to create different types of landscapes.

## Prior Work and Research

This project builds upon foundational work in procedural terrain generation:

### Terrain Generation Techniques
- Perlin Noise (Perlin, 1985) - A gradient noise function that produces naturally smooth variations, making it suitable for terrain heightmap generation
- Fractal Brownian Motion - A technique that combines multiple octaves of noise to create more natural-looking terrain features at different scales
- Heightmap-based Terrain - A common approach in computer graphics where terrain is represented as a 2D grid of height values

### Related Research
- "The Synthesis and Rendering of Eroded Fractal Terrains" (Musgrave et al., 1989) introduced the combination of fractal noise with erosion simulation
- "Terrain Generation Using Procedural Models Based on Hydrology" (Génevaux et al., 2013) demonstrated how hydraulic erosion principles can enhance terrain realism

## Algorithm and Theory

### Terrain Generation

The terrain generation uses Perlin noise as its foundation, which creates coherent pseudo-random values through the following process:

1. Define a grid where each vertex has an associated random gradient vector
2. For any point P:
   - Find the surrounding grid cell vertices
   - Calculate dot products between the gradient vectors and distance vectors to P
   - Interpolate the results using a smoothstep function

The noise function can be expressed as:

```
n(x,y) = lerp(
    lerp(dot(grad00, d00), dot(grad10, d10), smoothstep(fx)),
    lerp(dot(grad01, d01), dot(grad11, d11), smoothstep(fx)),
    smoothstep(fy)
)
```

where:
- gradXY are the gradient vectors at grid vertices
- dXY are the distance vectors from grid vertices to point P
- fx,fy are fractional coordinates within the grid cell
- smoothstep(t) = t³(6t² - 15t + 10) is used for smooth interpolation

This base noise is then combined using Fractal Brownian Motion (fBm). The height at any point (x, z) is calculated as:

$h(x,z) = \sum_{i=0}^{n-1} A^i \cdot \text{noise}(f^i \cdot x, f^i \cdot z)$

where:
- n = 6 (number of octaves)
- A = 0.5 (persistence)
- f = 2.0 (lacunarity)
- noise() is the Perlin noise function

The Perlin noise function provides gradient noise by:
1. Defining a grid of random gradients
2. Computing dot products between distance vectors and gradients
3. Interpolating between the dot products using a fade function:
   $f(t) = 6t^5 - 15t^4 + 10t^3$

This results in a heightmap H: ℝ² → [0,1] that is:
- Continuous and differentiable
- Exhibits self-similarity at different scales
- Has natural-looking variations

### Water Surface

The water surface is generated using a combination of procedural wave patterns and dynamic surface normals for realistic rendering:

1. Wave Generation:
   The height of the water surface at any point (x, z) is calculated as:
   
   $h(x,z,t) = \sum_{i=1}^{3} A_i \cdot \sin(f_i \cdot (k_{xi} \cdot x + k_{zi} \cdot z) + \omega_i \cdot t)$
   
   where:
   - $A_i$ are wave amplitudes (0.02, 0.015, 0.01)
   - $f_i$ are wave frequencies (10.0, 12.0, 14.0)
   - $k_{xi}, k_{zi}$ are directional wave numbers
   - $\omega_i$ are angular frequencies
   - t is time

2. Dynamic Normal Calculation:
   Surface normals are computed using partial derivatives:
   
   $N(x,z,t) = \text{normalize}(\vec{n})$ where $\vec{n} = (-\frac{\partial h}{\partial x}, 1.0, -\frac{\partial h}{\partial z})$

3. Fresnel Effect:
   Water reflectivity R at each point is calculated using the Fresnel equation:
   
   $R(\theta) = (1.0 - \cos(\theta))^4$
   
   where θ is the angle between the view direction and surface normal

## System Implementation

The project follows a layered architecture pattern with the following data flow:

```
User Input → Context Management → Scene Update → Render Pipeline
    ↑                                                |
    └──────────────────── Display ────────────────┘

1. User Input: Mouse/keyboard events for camera control
2. Context: Device and pipeline management
3. Scene Update:
   - Camera: Position and orientation updates
   - Terrain: Height generation, water simulation
4. Render Pipeline:
   - Height Pass: Terrain mesh rendering
   - Water Pass: Dynamic water surface
   - Skybox Pass: Environment rendering
5. Display: Frame presentation and vsync
```

The implementation follows a modular architecture with three main components:

1. **Context Management** (`src/context.rs`)
   - Handles WebGPU device and surface initialization
   - Manages window events and user input
   - Implements depth buffer and multi-pass rendering
   ```rust
   pub struct Context<'a> {
       surface: wgpu::Surface<'a>,
       device: wgpu::Device,
       pipelines: Vec<Box<dyn Renderable>>,
       depth_texture_view: wgpu::TextureView,
       camera: Camera,
   }
   ```
   - Pipeline execution order:
     ```rust
     // Order-dependent rendering passes
     height_pass.render(&mut render_pass, &camera);  
     water_pass.render(&mut render_pass, &camera);   
     skybox_pass.render(&mut render_pass, &camera);  
     ```

2. **Terrain System**
   
   2.1. **Height Generation** (`src/render/perlin.rs`)
   - Heightmap to mesh tessellation:
     ```rust
     // For an N×N heightmap, generate (N-1)×(N-1) quads
     for z in 0..N-1 {
         for x in 0..N-1 {
             vertices.extend_from_slice(&[
                 x,   heights[z][x],   z,    // Top-left
                 x+1, heights[z][x+1], z,    // Top-right
                 x,   heights[z+1][x], z+1,  // Bottom-left
                 // Second triangle...
             ]);
         }
     }
     ```
   - Counter-clockwise vertex ordering for back-face culling
   - Normal vectors computed from cross products
   
   2.2. **Water Surface** (`src/render/water.rs`)
   - Single quad mesh with dynamic displacement:
     ```rust
     let vertices = [
         [-SIZE_F32, WATER_LEVEL, -SIZE_F32],
         [ SIZE_F32, WATER_LEVEL, -SIZE_F32],
         [ SIZE_F32, WATER_LEVEL,  SIZE_F32],
         [-SIZE_F32, WATER_LEVEL,  SIZE_F32],
     ];
     ```
   - Transparency handling:
     ```rust
     depth_stencil: Some(wgpu::DepthStencilState {
         depth_write_enabled: false,
         depth_compare: wgpu::CompareFunction::LessEqual,
         ..Default::default()
     })
     ```
   - Shader effects (`water.wgsl`):
     - Multi-layered wave animation
     - Dynamic normal calculation
     - View-dependent Fresnel reflections
     - Continuous UV coordinate flow
   
   2.3. **Skybox** (`src/render/sky.rs`)
   - Inside-out cube rendering:
     ```rust
     vertices.extend_from_slice(&[
         pos.x, pos.y, pos.z,    // Front face
         pos.x, pos.y, -pos.z,   // with clockwise
         -pos.x, pos.y, -pos.z,  // winding order
     ]);
     pipeline_desc.primitive.cull_mode = None;
     ```
   - Special considerations:
     - Inverted winding order for internal rendering
     - Disabled back-face culling
     - Environment map sampling

3. **Camera System** (`src/render/camera.rs`)
   - First-person camera controls:
     - Position vector in world space
     - Pitch and yaw for orientation
     - Perspective projection matrix
   - Input handling:
     - Mouse: Look direction (yaw/pitch)
     - Keyboard: WASD movement
   - View matrix calculation:
     ```rust
     fn update_view_matrix(&mut self) {
         let (sin_pitch, cos_pitch) = pitch.sin_cos();
         let (sin_yaw, cos_yaw) = yaw.sin_cos();
         
         let forward = vec3(
             cos_pitch * cos_yaw,
             sin_pitch,
             cos_pitch * sin_yaw
         );
         
         let right = forward.cross(vec3(0.0, 1.0, 0.0));
         self.view = Mat4::look_to_rh(position, forward, right.cross(forward));
     }
     ```

The system architecture emphasizes real-time performance and interactivity:
```
Window Events → Context Update → Scene Update → Render Passes → Display
[Input] → [Device/Pipeline] → [Camera/Terrain] → [Height/Water/Sky] → [Present]
```

## Conclusion

This project successfully implemented a web-based terrain generation and visualization system using WebGPU, WebAssembly, and procedural noise techniques. The implementation demonstrates sophisticated graphics programming concepts including multi-pass rendering, compute shader-based terrain generation, and interactive 3D camera controls.

Key achievements include:
- Web-based 3D graphics implementation using Rust and WebAssembly
- Real-time terrain generation using compute shaders
- Multi-pass rendering system with depth testing and skybox
- Interactive camera system with mouse and keyboard controls

Future work could explore:
- Dynamic level of detail for improved performance
- Terrain texturing based on slope and height
- Advanced weather effects using compute shaders
- Hydraulic erosion simulation for more realistic terrain
- Improved sky rendering with dynamic time of day
