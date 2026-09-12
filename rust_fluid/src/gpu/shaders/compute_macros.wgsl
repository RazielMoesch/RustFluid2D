

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

@group(1) @binding(0) var<storage, read> f: array<f32>;
@group(1) @binding(1) var<storage, read_write> density: array<f32>;
@group(1) @binding(2) var<storage, read_write> velocity: array<vec2<f32>>;

const EX: array<f32, 9> = array<f32, 9>(0.0, 1.0, 0.0, -1.0, 0.0, 1.0, -1.0, -1.0, 1.0);
const EY: array<f32, 9> = array<f32, 9>(0.0, 0.0, 1.0, 0.0, -1.0, 1.0, 1.0, -1.0, -1.0);

@compute @workgroup_size(8, 8) 
fn compute_macros(@builtin(global_invocation_id) id: vec3<u32>) {

    let x = id.x;
    let y = id.y;
    
    if ( x >= uniforms.w || y >= uniforms.h) {
        return;
    }
    
    let cell_idx = y * uniforms.w + x;
    let idx = 9 * cell_idx;

    var local_density = 0.0;
    for ( var i = 0u; i < 9u; i += 1u ) {
        local_density += f[idx+i];
    }
    density[cell_idx] = local_density;

    var mx = 0.0;
    var my = 0.0;
    for ( var i = 0u; i < 9; i += 1 ) {
        mx += f[idx+i] * EX[i];
        my += f[idx+i] * EY[i];
    }

    let ux = mx / density[cell_idx];
    let uy = my / density[cell_idx];

    velocity[cell_idx] = vec2<f32>(ux, uy);

}

