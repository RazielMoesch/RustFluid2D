
struct Uniforms { // 32
    w: u32, // 4
    h: u32, // 4
    tau: f32, // 4
    u_lb: f32, // 4
    w_object: u32, // 4
    h_object: u32, // 4
    is_first_step: u32, // 4
    _pad: u32 // 4
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(1) @binding(0) var<storage, read_write> fa: array<f32>;
@group(1) @binding(1) var<storage, read_write> fb: array<f32>;
@group(1) @binding(2) var<storage, read> types: array<u32>;

const FLUID = 0u;
const SOLID = 1u;
const INLET = 2u;
const OUTLET = 3u;
const WEIGHTS: array<f32, 9> = array<f32, 9>(
    4.0/9.0, 1.0/9.0, 1.0/9.0, 1.0/9.0, 1.0/9.0, 1.0/36.0, 1.0/36.0, 1.0/36.0, 1.0/36.0
);
const EX: array<f32, 9> = array<f32, 9>(0.0, 1.0, 0.0, -1.0, 0.0, 1.0, -1.0, -1.0, 1.0);
const EY: array<f32, 9> = array<f32, 9>(0.0, 0.0, 1.0, 0.0, -1.0, 1.0, 1.0, -1.0, -1.0);
const get_opp: array<u32, 9> = array<u32, 9>(
    0u, 3u, 4u, 1u, 2u, 7u, 8u, 5u, 6u
);

@compute @workgroup_size(8, 8)
fn compute(@builtin(global_invocation_id) id: vec3<u32>) {

    let x = id.x; let y = id.y;
    let w = uniforms.w; let h = uniforms.h;
    if (x >= w || y >= h) { return; }

    let cell_idx = y * w + x;
    let idx = cell_idx * 9u;
    let cell_type = types[cell_idx];

    if (cell_type == SOLID) {
        for (var i = 0u; i < 9u; i += 1u) {
            fa[idx+i] = 0.0;
        }
        return;
    }

    // 1. STREAMING (Execute for all active cells first)
    for (var i = 0u; i < 9u; i += 1u) {
        var xsrc = i32(x) - i32(EX[i]);
        var ysrc = i32(y) - i32(EY[i]);

        if (ysrc >= i32(h)) { ysrc = i32(h) - 1; } else if (ysrc < 0) { ysrc = 0; }
        if (xsrc >= i32(w)) { xsrc = i32(w) - 1; } else if (xsrc < 0) { xsrc = 0; }

        let src_idx = u32(ysrc) * w + u32(xsrc);
        
        if (types[src_idx] == SOLID) {
            fb[idx+i] = fa[idx + get_opp[i]]; // Bounce-back
        } else {
            fb[idx+i] = fa[src_idx * 9u + i]; // Standard streaming
        }
    }

    // 2. BOUNDARY CONDITIONS (Apply to the newly streamed fb values)
    if (cell_type == OUTLET) {
        let inner_idx = 9u * (y * w + x - 1u);
        for (var i = 0u; i < 9u; i += 1u) {
            fb[idx + i] = fb[inner_idx + i]; 
            fa[idx + i] = fb[idx + i]; // Outlets skip collision
        }
        return;
    } 
    else if (cell_type == INLET) {
        let u_lb = uniforms.u_lb;
        let f0 = fb[idx + 0u]; let f2 = fb[idx + 2u]; let f4 = fb[idx + 4u];
        let f3 = fb[idx + 3u]; let f6 = fb[idx + 6u]; let f7 = fb[idx + 7u];

        let rho_in = (1.0 / (1.0 - u_lb)) * (f0 + f2 + f4 + 2.0 * (f3 + f6 + f7));

        fb[idx + 1u] = f3 + (2.0 / 3.0) * rho_in * u_lb;
        fb[idx + 5u] = f7 - 0.5 * (f2 - f4) + (1.0 / 6.0) * rho_in * u_lb;
        fb[idx + 8u] = f6 + 0.5 * (f2 - f4) + (1.0 / 6.0) * rho_in * u_lb;
    }

    // 3. COLLISION
    var density = 0.0;
    var mx = 0.0; var my = 0.0;
    for (var i = 0u; i < 9u; i += 1u) {
        density += fb[idx + i];
        mx += fb[idx+i] * EX[i];
        my += fb[idx+i] * EY[i];
    }

    let ux = select(0.0, mx / density, density > 0.0005);
    let uy = select(0.0, my / density, density > 0.0005);
    let u_squared = ux * ux + uy * uy;
    let omega = 1.0 / uniforms.tau;

    for (var i = 0u; i < 9u; i += 1u) {
        let eu = EX[i] * ux + EY[i] * uy;
        let feq = WEIGHTS[i] * density * (1.0 + 3.0 * eu + 4.5 * (eu * eu) - 1.5 * u_squared);
        fa[idx+i] = fb[idx+i] - omega * (fb[idx+i] - feq);
    }


}












