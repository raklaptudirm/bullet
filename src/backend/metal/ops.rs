use std::mem;
use std::slice;

use metal_rs::{Function, Library, MTLResourceOptions, MTLSize, NSUInteger};

use crate::loader::Feat;

use super::DeviceHandles;

#[derive(Clone)]
pub(crate) struct Kernels {
    backprop_relu: Function,
    backprop_crelu: Function,
    backprop_screlu: Function,
    activate_relu: Function,
    activate_crelu: Function,
    activate_screlu: Function,
    add_to: Function,
    sigmoid_mpe: Function,
}

impl Kernels {
    pub(crate) fn new(lib: &Library) -> Kernels {
        Self {
            backprop_relu: lib.get_function("backpropReLU", None).unwrap(),
            backprop_crelu: lib.get_function("backpropCReLU", None).unwrap(),
            backprop_screlu: lib.get_function("backpropSCReLU", None).unwrap(),
            activate_relu: lib.get_function("activateReLU", None).unwrap(),
            activate_crelu: lib.get_function("activateCReLU", None).unwrap(),
            activate_screlu: lib.get_function("activateSCReLU", None).unwrap(),
            add_to: lib.get_function("addTo", None).unwrap(),
            sigmoid_mpe: lib.get_function("sigmoidMPE", None).unwrap(),
        }
    }
}

pub unsafe fn splat_mul_matrix_vector(
    handle: &DeviceHandles,
    m: usize,
    n: usize,
    a_ptr: *const f32,
    x_ptr: *const f32,
    y_ptr: *mut f32,
    batch_size: usize,
) {
    unimplemented!()
}

pub unsafe fn splat_mul_matrixt_vector(
    handle: &DeviceHandles,
    m: usize,
    n: usize,
    a_ptr: *const f32,
    y_ptr: *const f32,
    x_ptr: *mut f32,
    batch_size: usize,
) {
    unimplemented!()
}

pub unsafe fn reduce_add_mul_vector_vectort(
    handle: &DeviceHandles,
    m: usize,
    n: usize,
    y_ptr: *const f32,
    x_ptr: *const f32,
    a_ptr: *mut f32,
    batch_size: usize,
) {
    unimplemented!()
}

pub unsafe fn reduce_add(
    handle: &DeviceHandles,
    _: *const f32,
    batch_size: usize,
    out_size: usize,
    inp: *const f32,
    out: *mut f32,
) {
    unimplemented!()
}

pub unsafe fn select(
    _: &DeviceHandles,
    batch_size: usize,
    input_size: usize,
    output_size: usize,
    buckets: *const u8,
    inp: *const f32,
    out: *mut f32,
) {
    unimplemented!();
}

pub unsafe fn select_backprop(
    _: &DeviceHandles,
    batch_size: usize,
    input_size: usize,
    output_size: usize,
    buckets: *const u8,
    inp: *const f32,
    out: *mut f32,
) {
    unimplemented!();
}

macro_rules! two_buffer_kernel {
    ($func:ident) => {
        pub unsafe fn $func(handle: &DeviceHandles, size: usize, inp: *const f32, out: *mut f32) {
            println!("starting pipeline");
            let pipeline = handle.device.new_compute_pipeline_state_with_function(&handle.kernels.$func).unwrap();

            let buffer_size = size.clone() as NSUInteger;

            println!("initializing size buffer");

            let siz_buffer = handle.device.new_buffer_with_data(
                &size as *const _ as *const std::ffi::c_void,
                mem::size_of::<usize>() as NSUInteger,
                MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeShared
            );

            println!("initializing other buffers");
            let inp_buffer = handle.device.new_buffer_with_data(
                unsafe { mem::transmute(inp) },
                buffer_size * mem::size_of::<f32>() as NSUInteger,
                MTLResourceOptions::StorageModeShared
            );
            let out_buffer = handle.device.new_buffer_with_data(
                unsafe { mem::transmute(out) },
                buffer_size * mem::size_of::<f32>() as NSUInteger,
                MTLResourceOptions::StorageModeShared
            );

            println!("starting command queue");

            let command_queue = handle.device.new_command_queue();
            let command_buffer = command_queue.new_command_buffer();
            let compute_encoder = command_buffer.new_compute_command_encoder();
            compute_encoder.set_compute_pipeline_state(&pipeline);
            compute_encoder.set_buffers(
                0,
                &[Some(&siz_buffer), Some(&inp_buffer), Some(&out_buffer)],
                &[0; 3]
            );

            let grid_size = MTLSize::new(buffer_size, 1, 1);
            let threadgroup_size = MTLSize::new(buffer_size, 1, 1);
            compute_encoder.dispatch_threads(grid_size, threadgroup_size);

            println!("commiting");

            compute_encoder.end_encoding();
            command_buffer.commit();

            println!("waiting");

            command_buffer.wait_until_completed();

            let data_ptr = out_buffer.contents();
            let mut out_slice = unsafe { slice::from_raw_parts_mut(out, buffer_size as usize) };
            out_slice.copy_from_slice(unsafe { slice::from_raw_parts_mut(data_ptr as *mut f32, buffer_size as usize) });
        }
    };
}

two_buffer_kernel!(backprop_relu);
two_buffer_kernel!(backprop_crelu);
two_buffer_kernel!(backprop_screlu);

two_buffer_kernel!(activate_relu);
two_buffer_kernel!(activate_crelu);
two_buffer_kernel!(activate_screlu);

two_buffer_kernel!(add_to);

pub unsafe fn sigmoid_mpe(
    handle: &DeviceHandles,
    buffer_size: usize,
    outputs: *mut f32,
    results: *const f32,
    errors: *mut f32,
    power: f32,
) {
    unimplemented!()
}

pub unsafe fn sparse_affine_forward(
    handle: &DeviceHandles,
    batch_size: usize,
    max_input_size: usize,
    output_size: usize,
    weights: *const f32,
    biases: *const f32,
    inputs: *const Feat,
    outputs: *mut f32,
) {
    unimplemented!()
}

pub unsafe fn sparse_affine_backward(
    handle: &DeviceHandles,
    batch_size: usize,
    max_active_inputs: usize,
    input_size: usize,
    output_size: usize,
    weights_grad: *mut f32,
    biases_grad: *mut f32,
    inputs: *const Feat,
    errors: *const f32,
    output: *const f32,
    ft_reg: f32,
) {
    unimplemented!()
}

pub unsafe fn single_sparse_affine_forward(
    handle: &DeviceHandles,
    batch_size: usize,
    max_active_inputs: usize,
    output_size: usize,
    weights: *const f32,
    biases: *const f32,
    inputs: *const Feat,
    outputs: *mut f32,
) {
    unimplemented!()
}

pub unsafe fn single_sparse_affine_backward(
    handle: &DeviceHandles,
    batch_size: usize,
    max_active_inputs: usize,
    input_size: usize,
    output_size: usize,
    weights_grad: *mut f32,
    biases_grad: *mut f32,
    inputs: *const Feat,
    errors: *const f32,
    output: *const f32,
    ft_reg: f32,
) {
    unimplemented!()
}

pub unsafe fn splat_add(handle: &DeviceHandles, batch_size: usize, tensor_size: usize, inp: *const f32, out: *mut f32) {
    unimplemented!()
}

pub unsafe fn update_weights(
    handle: &DeviceHandles,
    network_size: usize,
    decay: f32,
    adj: f32,
    rate: f32,
    network: *mut f32,
    momentum: *mut f32,
    velocity: *mut f32,
    gradients: *const f32,
) {
    unimplemented!()
}