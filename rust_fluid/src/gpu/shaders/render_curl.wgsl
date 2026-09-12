struct Uniforms {
    w: u32,
    h: u32,
    tau: f32,
    u_lb: f32,
    w_object: u32,
    h_object: u32,
    is_first_step: u32,
    _pad: u32
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

fn vorticity_colormap(val: f32) -> vec3<f32> {
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

    let x = u32( uv.x * f32(uniforms.w) );
    let y = u32( uv.y * f32(uniforms.h) );
    
    let cx = clamp(x, 0, uniforms.w - 1u);
    let cy = clamp(y, 0, uniforms.h - 1u);
    let cell_idx = cy * uniforms.w + cx;

    if ( types[cell_idx] == 1u ) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    
    let cx_east = clamp(cx + 1u, 0, uniforms.w - 1u);
    let cx_west = clamp(cx - 1u, 0, uniforms.w - 1u);
    let cy_north = clamp(cy + 1u, 0, uniforms.h - 1u);
    let cy_south = clamp(cy - 1u, 0, uniforms.h - 1u);

    let vel_east = velocity[cy * uniforms.w + cx_east];
    let vel_west = velocity[cy * uniforms.w + cx_west];
    let vel_north = velocity[cy_north * uniforms.w + cx];
    let vel_south = velocity[cy_south * uniforms.w + cx];

    let duy_dx = (vel_east.y - vel_west.y) * 0.5;
    let dux_dy = (vel_north.x - vel_south.x) * 0.5;

    let curl = duy_dx - dux_dy;

    let scale = uniforms.u_lb * 0.15;
    let normalized_curl = curl / scale;

    return vec4<f32>(vorticity_colormap(normalized_curl), 1.0);
}