
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
@group(1) @binding(0) var<storage, read> velocity: array<vec2<f32>>;
@group(1) @binding(1) var<storage, read_write> tracers: array<Tracer>;

@compute @workgroup_size(64)
fn compute_tracers(@builtin(global_invocation_id) id: vec3<u32>) {
    let tracer_idx = id.x;
    let num_tracers = arrayLength(&tracers);

    if (tracer_idx >= num_tracers) {
        return;
    }

    var pos = tracers[tracer_idx].pos;

    if (uniforms.is_first_step == 1u) {
        tracers[tracer_idx].prev_pos = pos;
    }

    if (pos.x >= f32(uniforms.w) - 2.0 || pos.x < 1.0) {
        pos.x = 2.0;
        tracers[tracer_idx].pos = pos;
        tracers[tracer_idx].prev_pos = pos;
        return;
    }

    if (pos.y >= f32(uniforms.h) - 2.0 || pos.y < 1.0) {
        pos.x = 2.0;
        tracers[tracer_idx].pos = pos;
        tracers[tracer_idx].prev_pos = pos;
        return;
    }

    let x0 = u32(floor(pos.x));
    let y0 = u32(floor(pos.y));
    let x1 = x0 + 1u;
    let y1 = y0 + 1u;

    let wx = pos.x - f32(x0);
    let wy = pos.y - f32(y0);

    let v00 = velocity[y0 * uniforms.w + x0];
    let v10 = velocity[y0 * uniforms.w + x1];
    let v01 = velocity[y1 * uniforms.w + x0];
    let v11 = velocity[y1 * uniforms.w + x1];

    let v_bottom = mix(v00, v10, wx);
    let v_top = mix(v01, v11, wx);
    let v_interp = mix(v_bottom, v_top, wy);

    let new_pos = pos + v_interp;

    tracers[tracer_idx].pos = new_pos;

}
