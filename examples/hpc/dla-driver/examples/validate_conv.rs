#![no_std]
#![no_main]

extern crate alloc;

use dla_driver::utils::generate_output_tensor;
use dla_driver::*;
use headsail_bsp::apb_uart::ApbUart0;
use headsail_bsp::{
    init_heap, rt::entry, sprint, sprintln, tb::report_fail, tb::report_ok, tb::report_pass,
};
use panic_halt as _;

use alloc::vec::Vec;
use dla_driver::tensor3::{Order3, Tensor3};
use dla_driver::tensor4::{Order4, Tensor4};

fn validate_conv2d_tiny() -> bool {
    let mut uart = unsafe { ApbUart0::instance() };
    sprintln!("din\r\n");
    let din: Vec<i8> = uart.read_to_heap(75).into_iter().map(|x| x as i8).collect();

    sprintln!("wgt\r\n");
    let wgt: Vec<i8> = uart.read_to_heap(54).into_iter().map(|x| x as i8).collect();

    sprintln!("dout\r\n");
    let dout_i32: Vec<i32> = uart
        .read_to_heap(18)
        .into_iter()
        .map(|x| x as i8)
        .map(i32::from)
        .collect();

    sprintln!("Inputs read\r\n");

    let din_tensor: Tensor3<i8> = Tensor3::from_data_buffer(3, 5, 5, din, Order3::HWC).unwrap();
    let wgt_tensor: Tensor4<i8> = Tensor4::from_data_buffer(2, 3, 3, 3, wgt, Order4::HWKC).unwrap();
    let dout_tensor =
        generate_output_tensor(&din_tensor, &wgt_tensor, dout_i32, Order3::HWC, None, None);
    let mut output: Tensor3<i32> =
        dla_driver::layers::conv2d(din_tensor, wgt_tensor, None, None, None, None, None);
    output.permute(Order3::HWC);

    sprint!("\ndla out | dout\n");
    let dout_tensor_buf = dout_tensor.to_buffer();
    let output_tensor_buf = output.to_buffer();

    dout_tensor_buf == output_tensor_buf
}

fn validate_conv2d() -> bool {
    let mut uart = unsafe { ApbUart0::instance() };
    sprintln!("din\r\n");
    let din: Vec<i8> = uart
        .read_to_heap(4096)
        .into_iter()
        .map(|x| x as i8)
        .collect();

    sprintln!("wgt\r\n");
    let wgt: Vec<i8> = uart
        .read_to_heap(2304)
        .into_iter()
        .map(|x| x as i8)
        .collect();

    sprintln!("dout\r\n");
    let dout: Vec<u8> = uart.read_to_heap(3136 * 4);
    let mut dout_i32: Vec<i32> = Vec::with_capacity(3136);

    for chunk in dout.chunks(4) {
        let mut value: u32 = 0;
        for (i, &byte) in chunk.iter().rev().enumerate() {
            value |= (byte as u32) << (8 * i);
        }
        dout_i32.push(value as i32);
    }

    let din_tensor: Tensor3<i8> = Tensor3::from_data_buffer(16, 16, 16, din, Order3::HWC).unwrap();
    let wgt_tensor: Tensor4<i8> =
        Tensor4::from_data_buffer(16, 16, 3, 3, wgt, Order4::HWKC).unwrap();
    let dout_tensor =
        generate_output_tensor(&din_tensor, &wgt_tensor, dout_i32, Order3::HWC, None, None);

    let mut output =
        dla_driver::layers::conv2d(din_tensor, wgt_tensor, None, None, None, None, None);
    output.permute(Order3::HWC);

    sprint!("\n");
    let dout_tensor_buf = dout_tensor.to_buffer();
    let output_tensor_buf = output.to_buffer();

    dout_tensor_buf == output_tensor_buf
}

fn validate_conv2d_bias() -> bool {
    let mut uart = unsafe { ApbUart0::instance() };
    sprintln!("din\r\n");
    let din: Vec<i8> = uart
        .read_to_heap(4096)
        .into_iter()
        .map(|x| x as i8)
        .collect();

    sprintln!("wgt\r\n");
    let wgt: Vec<i8> = uart
        .read_to_heap(2304)
        .into_iter()
        .map(|x| x as i8)
        .collect();

    sprintln!("dout\r\n");
    let dout: Vec<i8> = uart
        .read_to_heap(1024)
        .into_iter()
        .map(|x| x as i8)
        .collect();

    sprintln!("bias\r\n");
    let bias: Vec<u8> = uart.read_to_heap(16 * 2);

    let mut bias_i16: Vec<i16> = Vec::with_capacity(16);
    for chunk in bias.chunks(2) {
        let mut value: u16 = 0;
        for (i, &byte) in chunk.iter().rev().enumerate() {
            value |= (byte as u16) << (8 * i);
        }
        bias_i16.push(value as i16);
    }

    let din_tensor: Tensor3<i8> = Tensor3::from_data_buffer(16, 16, 16, din, Order3::HWC).unwrap();
    let wgt_tensor: Tensor4<i8> =
        Tensor4::from_data_buffer(16, 16, 3, 3, wgt, Order4::HWKC).unwrap();

    let padding = Padding {
        top: 0,
        left: 0,
        right: 1,
        bottom: 1,
        padding_value: 0,
    };

    let stride = Stride { x: 2, y: 2 };

    let dout_tensor = generate_output_tensor(
        &din_tensor,
        &wgt_tensor,
        dout,
        Order3::HWC,
        Some(padding.clone()),
        Some(stride.clone()),
    );

    let mut output = dla_driver::layers::conv2d_bias(
        din_tensor,
        wgt_tensor,
        bias_i16,
        Some(padding),
        Some(stride),
        Some(6),
        Some(4),
        None,
    );

    output.permute(Order3::HWC);

    let dout_tensor_buf = dout_tensor.to_buffer();
    let output_tensor_buf = output.to_buffer();

    for (i, x) in dout_tensor.to_buffer().into_iter().enumerate() {
        if x != output_tensor_buf[i] {
            sprintln!(
                "Error at pos {}! Expected {} got {} instead",
                i,
                x,
                output_tensor_buf[i]
            )
        }
    }
    dout_tensor_buf == output_tensor_buf
}

#[entry]
fn main() -> ! {
    // SAFETY: `init_heap` must be called once only
    unsafe { init_heap() };
    let mut _uart = ApbUart0::init(30_000_000, 115_200);
    sprintln!("Validate conv2d");
    let mut succesful_test = 0;

    if validate_conv2d_tiny() {
        report_ok();
        sprintln!(" Tiny test succesful");
        succesful_test += 1;
    } else {
        report_fail();
        sprintln!(" Tiny test failed");
    }

    if validate_conv2d() {
        report_ok();
        sprintln!(" 16x16x16_3x3 conv2d test succesful");
        succesful_test += 1;
    } else {
        report_fail();
        sprintln!(" 16x16x16_3x3 conv2d test failed");
    }

    if validate_conv2d_bias() {
        report_ok();
        sprintln!(" 16x16x16_3x3 conv2d bias test succesful");
        succesful_test += 1;
    } else {
        report_fail();
        sprintln!(" 16x16x16_3x3 conv2d bias test failed");
    }

    if succesful_test == 3 {
        report_pass();
        sprintln!(" All tests succesful!\r\n");
    } else {
        report_fail();
        sprintln!(" Not all tests succesful!\r\n");
    }

    loop {}
}
