

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

struct Tracer {
    pos: vec2<f32>,
    prev_pos: vec2<f32>
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(1) @binding(0) var<storage, read_write> fa: array<f32>;
@group(1) @binding(1) var<storage, read_write> fb: array<f32>; 
@group(1) @binding(2) var<storage, read_write> types: array<u32>;
@group(1) @binding(3) var<storage, read_write> density: array<f32>;
@group(1) @binding(4) var<storage, read_write> velocity: array<vec2<f32>>;
@group(1) @binding(5) var<storage, read> object: array<u32>;
@group(1) @binding(6) var<storage, read_write> tracers: array<Tracer>;


const WEIGHTS: array<f32, 9> = array<f32, 9>(
    4.0/9.0, 1.0/9.0, 1.0/9.0, 1.0/9.0, 1.0/9.0, 1.0/36.0, 1.0/36.0, 1.0/36.0, 1.0/36.0
);
const EX: array<f32, 9> = array<f32, 9>(0.0, 1.0, 0.0, -1.0, 0.0, 1.0, -1.0, -1.0, 1.0);
const EY: array<f32, 9> = array<f32, 9>(0.0, 0.0, 1.0, 0.0, -1.0, 1.0, 1.0, -1.0, -1.0);


@compute @workgroup_size(8, 8)
fn compute_initial_conditions( @builtin(global_invocation_id) id: vec3<u32> ) {

    let x = id.x;
    let y = id.y;
    
    if ( x >= uniforms.w || y >= uniforms.h) {
        return;
    }
    
    let cell_idx = y * uniforms.w + x;
    let f_idx = 9 * cell_idx;

    var cell_type = 0u;

    if (x == 0u) {
        cell_type = 2u; 
    } else if (x == uniforms.w - 1u) {
        cell_type = 3u;
    } else if (y == 0u || y == uniforms.h - 1u) {
        cell_type = 1u; 
    }

    let obj_start_x = (uniforms.w * 25u) / 100u;
    let obj_start_y = (uniforms.h - uniforms.h_object) / 2u;

    if (x >= obj_start_x && x < obj_start_x + uniforms.w_object &&
        y >= obj_start_y && y < obj_start_y + uniforms.h_object) {
        
        let local_x = x - obj_start_x;
        let local_y = y - obj_start_y;
        let obj_idx = local_y * uniforms.w_object + local_x;
        
        if (object[obj_idx] == 1u) {
            cell_type = 1u;
        }
    }
    
    types[cell_idx] = cell_type;


    density[cell_idx] = 1.0;


    if (cell_type == 2u || cell_type == 1u) {
        velocity[cell_idx] = vec2<f32>(uniforms.u_lb, 0.0);
    } else {
        velocity[cell_idx] = vec2<f32>(uniforms.u_lb, 0.0);
    }

    let u = velocity[cell_idx];
    let u_squared = dot(u, u);

    for ( var i = 0u; i < 9; i += 1 ) {
        let eu = EX[i] * u.x + EY[i] * u.y;
        let feq = WEIGHTS[i] * (1.0 + 3.0 * eu + 4.5 * eu * eu - 1.5 * u_squared);
        fa[f_idx + i] = feq;
        fb[f_idx + i] = feq;
    }

    let num_tracers = arrayLength(&tracers);
    if (cell_idx < num_tracers) {
        let cols = max(1u, u32(ceil(sqrt(f32(num_tracers)))));
        let rows = max(1u, (num_tracers + cols - 1u) / cols);
        let col = cell_idx % cols;
        let row = cell_idx / cols;
        let x_step = f32(uniforms.w) / f32(cols + 1u);
        let y_step = f32(uniforms.h) / f32(rows + 1u);
        let start_x = x_step * f32(col + 1u);
        let start_y = y_step * f32(row + 1u);

        tracers[cell_idx] = Tracer(vec2<f32>(start_x, start_y), vec2<f32>(start_x, start_y));
    }



}








