


use bytemuck::{Pod, Zeroable};


#[repr(C)]
#[derive(Zeroable, Pod, Clone, Copy)]
pub struct Tracer {
    pub pos: [f32; 2],
    pub prev_pos: [f32; 2]
}


#[repr(C)]
#[derive(Zeroable, Pod, Clone, Copy)]
pub struct Uniforms {
    pub w: u32,
    pub h: u32,
    pub tau: f32,
    pub u_lb: f32,
    pub w_object: u32,
    pub h_object: u32,
    pub is_first_step: u32,
    pub _pad: u32
}

