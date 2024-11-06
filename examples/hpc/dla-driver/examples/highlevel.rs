#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

use headsail_bsp::{init_heap, rt::entry, sprint, sprintln};
use panic_halt as _;

use dla_driver::tensor3::{Order3, Tensor3};
use dla_driver::tensor4::{Order4, Tensor4};

use alloc::vec::Vec;

fn dense_test() {
    sprintln!("dense_test: enter");
    let din: Vec<i8> = vec![
        1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5,
    ];
    let din_tensor: Tensor3<i8> = Tensor3::from_data_buffer(1, 5, 5, din, Order3::CHW).unwrap();

    let dense_weight_size = din_tensor.height() * din_tensor.width() * din_tensor.channels() * 5;

    let mut weight: Vec<i8> = vec![];
    for _ in 0..dense_weight_size {
        weight.push(1)
    }

    dla_driver::layers::dense(5, din_tensor, weight);
    sprintln!("dense_test: leave");
}

fn conv_test() {
    sprintln!("conv_test: enter");
    let din: Vec<i8> = vec![
        0, 0, 0, 2, 0, 0, 1, 2, 1, 2, 0, 0, 1, 2, 0, 1, 0, 0, 0, 2, 0, 0, 1, 0, 1, 2, 0, 1, 0, 1,
        0, 0, 2, 2, 1, 1, 0, 2, 1, 1, 2, 1, 2, 2, 1, 0, 0, 1, 1, 2, 0, 1, 1, 1, 0, 0, 2, 0, 1, 2,
        1, 0, 0, 1, 2, 1, 1, 1, 0, 0, 1, 1, 2, 0, 2,
    ];
    let wgt: Vec<i8> = vec![
        -1, -1, 0, -1, 0, 0, -1, -1, 1, 0, 0, 1, 1, -1, -1, 1, -1, 0, 1, 0, -1, -1, 1, -1, -1, 0,
        -1, 1, 0, 0, -1, 0, 1, 0, -1, 1, 0, 1, -1, -1, 0, 0, 0, -1, -1, 0, -1, 1, -1, -1, -1, 0, 1,
        0,
    ];
    let dout: Vec<i32> = vec![
        1, -10, -8, -3, -6, -5, -2, -7, -10, -2, -4, -10, -7, 0, -3, -7, -2, -1,
    ];

    let din_tensor: Tensor3<i8> = Tensor3::from_data_buffer(3, 5, 5, din, Order3::CHW).unwrap();
    let wgt_tensor: Tensor4<i8> = Tensor4::from_data_buffer(2, 3, 3, 3, wgt, Order4::KCHW).unwrap();
    let _dout_tensor: Tensor3<i32> = Tensor3::from_data_buffer(2, 3, 3, dout, Order3::CHW).unwrap();

    let mut output: Tensor3<i8> =
        dla_driver::layers::conv2d(din_tensor, wgt_tensor, None, None, None, None, None);
    output.permute(Order3::CWH);
    sprintln!("conv_test: leave");
}

#[entry]
fn main() -> ! {
    // SAFETY: `init_heap` must be called once only
    unsafe { init_heap() };
    dense_test();
    conv_test();

    loop {}
}
