struct Uniforms {
    w: u32,
    h: u32,
    tau: f32,
    u_lb: f32,
    w_object: u32,
    h_object: u32,
    is_first_step: u32,
    obj_x: u32,
    obj_y: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
};

struct Tracer {
    pos: vec2<f32>,
    prev_pos: vec2<f32>
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var<storage, read> tracers: array<Tracer>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vs(
    @builtin(vertex_index) vi: u32,
    @builtin(instance_index) ii: u32
) -> VertexOutput {
    let quad = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5), vec2<f32>( 0.5, -0.5), vec2<f32>(-0.5,  0.5),
        vec2<f32>(-0.5,  0.5), vec2<f32>( 0.5, -0.5), vec2<f32>( 0.5,  0.5)
    );

    let local_uv = quad[vi];
    let point_radius = 9.0;
    let offset = local_uv * point_radius;
    let p = tracers[ii].pos + offset;

    let screen_x = (p.x / f32(uniforms.w)) * 2.0 - 1.0;
    let screen_y = 1.0 - (p.y / f32(uniforms.h)) * 2.0;

    var output: VertexOutput;
    output.position = vec4<f32>(screen_x, screen_y, 0.0, 1.0);
    output.uv = local_uv;
    return output;
}

@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.uv);
    if (dist > 1.0) {
        discard;
    }

    let glow = 1.0 - smoothstep(0.0, 1.0, dist);
    let alpha = 1.0 - smoothstep(0.65, 1.0, dist);
    let color = vec3<f32>(0.08, 0.82, 1.0);
    return vec4<f32>(mix(color, vec3<f32>(1.0, 0.92, 0.65), 0.35 * glow), alpha * 0.95);
}