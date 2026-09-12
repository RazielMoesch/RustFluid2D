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
@group(1) @binding(0) var<storage, read> density: array<f32>;
@group(1) @binding(1) var<storage, read> types: array<u32>;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vs( @builtin(vertex_index) vidx: u32 ) -> VOut {

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

fn density_colormap(t: f32) -> vec3<f32> {
    let v = clamp(t, 0.0, 1.0);

    let c0 = vec3<f32>(0.10, 0.05, 0.30);
    let c1 = vec3<f32>(0.55, 0.10, 0.55);
    let c2 = vec3<f32>(0.95, 0.35, 0.35);
    let c3 = vec3<f32>(1.00, 0.80, 0.20);

    if (v < 0.33) {
        return mix(c0, c1, v / 0.33);
    } else if (v < 0.66) {
        return mix(c1, c2, (v - 0.33) / 0.33);
    } else {
        return mix(c2, c3, (v - 0.66) / 0.34);
    }
}

@fragment
fn fs( @location(0) uv: vec2<f32> ) -> @location(0) vec4<f32> {
    
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

    let d = density[cell_idx];
    let min_d = 0.98;
    let max_d = 1.02;

    let range = max(max_d - min_d, 1e-8);
    let normalized = clamp((d - min_d) / range, 0.0, 1.0);

    let color = density_colormap(normalized);

    return vec4<f32>(color, 1.0);
}