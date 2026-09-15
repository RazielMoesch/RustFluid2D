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

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var<storage, read> velocity: array<vec2<f32>>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) speed: f32
};

@vertex
fn vs(
    @builtin(vertex_index) vi: u32,
    @builtin(instance_index) ii: u32
) -> VertexOutput {
    let step = 4u;
    let cols = (uniforms.w + step - 1u) / step;
    let rows = (uniforms.h + step - 1u) / step;
    let col = ii % cols;
    let row = ii / cols;
    if (row >= rows) {
        var out: VertexOutput;
        out.position = vec4<f32>(2.0, 2.0, 0.0, 1.0);
        out.uv = vec2<f32>(0.0, 0.0);
        out.speed = 0.0;
        return out;
    }

    let x = min(col * step + (step / 2u), uniforms.w - 1u);
    let y = min(row * step + (step / 2u), uniforms.h - 1u);
    let cell = y * uniforms.w + x;
    let vel = velocity[cell];
    let speed = length(vel);
    if (speed < 0.01) {
        var out: VertexOutput;
        out.position = vec4<f32>(2.0, 2.0, 0.0, 1.0);
        out.uv = vec2<f32>(0.0, 0.0);
        out.speed = 0.0;
        return out;
    }

    let dir = normalize(vel);
    let perp = vec2<f32>(-dir.y, dir.x);
    let quad = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.12), vec2<f32>(0.5, -0.12), vec2<f32>(-0.5,  0.12),
        vec2<f32>(-0.5,  0.12), vec2<f32>(0.5, -0.12), vec2<f32>(0.5,  0.12)
    );
    let local_uv = quad[vi];
    let arrow_len = clamp(speed * 16.0, 4.0, 14.0);
    let offset = (local_uv.x * arrow_len) * dir + (local_uv.y * 1.8) * perp;
    let center = vec2<f32>(f32(x) + 0.5, f32(y) + 0.5);
    let world = center + offset;

    let screen_x = (world.x / f32(uniforms.w)) * 2.0 - 1.0;
    let screen_y = 1.0 - (world.y / f32(uniforms.h)) * 2.0;

    var output: VertexOutput;
    output.position = vec4<f32>(screen_x, screen_y, 0.0, 1.0);
    output.uv = local_uv;
    output.speed = speed;
    return output;
}

@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let strength = clamp(in.speed / 2.5, 0.0, 1.0);
    let alpha = 1.0 - smoothstep(0.10, 0.9, abs(in.uv.y));
    let color = mix(vec3<f32>(0.15, 0.72, 1.0), vec3<f32>(1.0, 0.82, 0.18), strength);
    return vec4<f32>(color, alpha * 0.9);
}
