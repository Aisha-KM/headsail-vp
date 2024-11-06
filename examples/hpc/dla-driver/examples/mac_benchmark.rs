#![no_std]
#![no_main]

extern crate alloc;

use dla_driver::*;
use headsail_bsp::{init_heap, rt::entry, sprint, sprintln};
use panic_halt as _;

use rand::rngs::SmallRng;
use rand::RngCore;
use rand::SeedableRng;

use alloc::vec::Vec;

fn calculate_conv2d_out_param_dim(
    input: (u32, u32),
    kernel: (u32, u32),
    padding: (u32, u32),
    dilation: (u32, u32),
    stride: (u32, u32),
) -> (u32, u32) {
    let output_width = (input.0 + 2 * padding.0 - dilation.0 * (kernel.0 - 1) - 1) / stride.0 + 1;
    let output_height = (input.1 + 2 * padding.1 - dilation.1 * (kernel.1 - 1) - 1) / stride.1 + 1;
    (output_width, output_height)
}

fn generate_random_array(buffer: &mut [i8], size: usize) {
    let mut rng = SmallRng::seed_from_u64(1234567890);
    for i in 0..size {
        buffer[i] = rng.next_u64() as i8;
    }
}

fn generate_random_matrix(height: u32, width: u32, seed: u64) -> Vec<i8> {
    let mut res: Vec<i8> = Vec::new();
    let mut rng = SmallRng::seed_from_u64(seed);
    for _ in 0..(height * width) {
        res.push((rng.next_u64() & 0xFF) as i8);
    }
    res
}

fn generate_random_matrix_small(height: u32, width: u32, seed: u64) -> Vec<i8> {
    let mut res: Vec<i8> = Vec::new();
    let mut rng = SmallRng::seed_from_u64(seed);
    for _ in 0..(height * width) {
        res.push((rng.next_u64() & 0x3) as i8);
    }
    res
}

fn run_random_layer(
    dla: &mut Dla,
    input_width: u32,
    input_height: u32,
    kernel_width: u32,
    kernel_height: u32,
    seed: u64,
) -> Vec<i8> {
    let mut input = generate_random_matrix(input_width, input_height, seed);
    let mut kernel = generate_random_matrix_small(kernel_width, kernel_height, seed * 2);

    // Calculate output size
    let (output_width, output_height) = calculate_conv2d_out_param_dim(
        (input_width, input_height),
        (kernel_width, kernel_height),
        (0, 0),
        (1, 1),
        (1, 1),
    );

    let mut bias = generate_random_matrix(output_height, output_width, seed * 4);
    dla.write_data_bank(MEMORY_BANK_12_OFFSET, &mut bias);

    // Initalize layer
    let config = LayerConfig {
        input_bank: Some(MemoryBank::Bank0),
        kernel_bank: Some(MemoryBank::Bank4),
        output_bank: Some(MemoryBank::Bank8),
        bias_addr: Some((MEMORY_BANK_12_OFFSET + MEMORY_BANK_BASE_ADDR) as u32),
        pp_enabled: true,
        relu_enabled: true,
        bias_enabled: true,
        input_size: Some(InputSize {
            channels: 1,
            width: input_width,
            height: input_height,
        }),
        kernel_size: Some(KernelSize {
            s_channels: 1,
            kernels: 1,
            width: kernel_width,
            height: kernel_height,
        }),
        padding: Some(Padding {
            top: 0,
            right: 0,
            left: 0,
            bottom: 0,
            padding_value: 0,
        }),
        stride: Some(Stride { x: 1, y: 1 }),
        mac_clip: Some(8),
        pp_clip: Some(8),
        simd_mode: Some(SimdBitMode::EightBits),
    };

    dla.init_layer(config);

    // Write input and kernel to buffer
    dla.write_input(&mut input);
    dla.write_kernel(&mut kernel);

    // Mark data ready to start calculations
    dla.kernel_data_ready(true);
    dla.input_data_ready(true);

    // Print the matrix
    sprintln!("Waiting for calculation");
    while !dla.handle_handshake() {}
    sprintln!("Calculation ready");
    dla.read_output_i8(output_width as usize * output_height as usize * 16)
}

#[entry]
fn main() -> ! {
    // SAFETY: `init_heap` must be called once only
    unsafe { init_heap() };

    let mut dla = Dla::new();
    sprintln!("Starting benchmark..");

    for x in 0..2 {
        let res = run_random_layer(&mut dla, 8, 8, 2, 2, x * x);
        for x in res {
            sprint!("{:?} ", x);
        }
        sprint!("\n\n");
    }
    loop {}
}
