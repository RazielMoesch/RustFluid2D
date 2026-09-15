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
@group(1) @binding(1) var<storage, read> types: array<u32>;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vs(@builtin(vertex_index) vidx: u32) -> VOut {

    var out: VOut;
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0)
    );
    let uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0)
    );

    out.pos = vec4<f32>(pos[vidx], 0.0, 1.0);
    out.uv = uvs[vidx];

    return out;
}

fn diverging_colormap(val: f32) -> vec3<f32> {
    let v = clamp(val, -1.0, 1.0);

    let neg_color = vec3<f32>(0.00, 0.45, 0.95);
    let zero_color = vec3<f32>(0.04, 0.06, 0.10);
    let pos_color = vec3<f32>(1.00, 0.20, 0.10);

    if (v < 0.0) {
        return mix(zero_color, neg_color, -v);
    } else {
        return mix(zero_color, pos_color, v);
    }
}

@fragment
fn fs(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {

    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        discard;
    }

    let x = u32(uv.x * f32(uniforms.w));
    let y = u32(uv.y * f32(uniforms.h));
    let cx = clamp(x, 0, uniforms.w - 1u);
    let cy = clamp(y, 0, uniforms.h - 1u);
    let cell_idx = cy * uniforms.w + cx;

    if ( types[cell_idx] == 1u ) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let x_vel = velocity[cell_idx][0];
    let max_vel = uniforms.u_lb * 1.5;

    let norm_x_vel = clamp(x_vel / max_vel, -1.0, 1.0);
    return vec4<f32>( diverging_colormap(norm_x_vel), 1.0 );
}