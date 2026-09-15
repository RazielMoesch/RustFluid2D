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

fn fluid_colormap(t: f32) -> vec3<f32> {
    let v = clamp(t, 0.0, 1.0);

    let c0 = vec3<f32>(0.02, 0.04, 0.12);
    let c1 = vec3<f32>(0.05, 0.35, 0.75);
    let c2 = vec3<f32>(0.00, 0.80, 0.70);
    let c3 = vec3<f32>(0.95, 0.85, 0.20);
    let c4 = vec3<f32>(1.00, 0.25, 0.05);

    if (v < 0.25) {
        return mix(c0, c1, v / 0.25);
    } else if (v < 0.50) {
        return mix(c1, c2, (v - 0.25) / 0.25);
    } else if (v < 0.75) {
        return mix(c2, c3, (v - 0.50) / 0.25);
    } else {
        return mix(c3, c4, (v - 0.75) / 0.25);
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
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    }

    let ux = velocity[cell_idx][0];
    let uy = velocity[cell_idx][1];

    let speed = sqrt( ux * ux + uy * uy  );

    let max_speed = uniforms.u_lb * 1.5;

    let norm_speed = clamp(speed / max_speed, 0.0, 1.0);

    return vec4<f32>(fluid_colormap(norm_speed), 1.0);
}