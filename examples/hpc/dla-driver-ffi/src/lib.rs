//! # DLA driver FFI
//!
//! Makes DLA's highlevel API availeable from C via FFI.
#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use core::ffi::{c_char, CStr};
use core::slice;
use dla_driver::layers::{conv2d, conv2d_bias, conv2d_bias_relu, conv2d_relu};
use dla_driver::tensor3::{rescale, Order3, Tensor3};
use dla_driver::tensor4::{Order4, Tensor4};
use dla_driver::{Padding, Stride};

/// Converts C-types to DLA Tensors for use with the highlevel layer
unsafe fn ffi_data_import(
    input_data: *const i8,
    input_channels: usize,
    input_height: usize,
    input_width: usize,
    input_order: *const c_char,
    kernel_data: *const i8,
    kernel_amount: usize,
    kernel_channels: usize,
    kernel_height: usize,
    kernel_width: usize,
    kernel_order: *const c_char,
) -> (Tensor3<i8>, Tensor4<i8>) {
    let mut input_data: Vec<i8> = unsafe {
        slice::from_raw_parts(input_data, input_channels * input_height * input_width).to_vec()
    };

    let input_order_string = unsafe { CStr::from_ptr(input_order).to_str().unwrap_unchecked() };
    let input_tensor = unsafe {
        Tensor3::from_data_buffer(
            input_channels,
            input_height,
            input_width,
            input_data,
            Order3::try_from(input_order_string).unwrap_unchecked(),
        )
        .unwrap_unchecked()
    };

    let kernels_data: Vec<i8> = unsafe {
        slice::from_raw_parts(
            kernel_data,
            kernel_amount * kernel_channels * kernel_height * kernel_width,
        )
        .to_vec()
    };

    let kernel_order_string = unsafe { CStr::from_ptr(kernel_order).to_str().unwrap_unchecked() };
    let kernels_tensor = unsafe {
        Tensor4::from_data_buffer(
            kernel_amount,
            kernel_channels,
            kernel_height,
            kernel_width,
            kernels_data,
            Order4::try_from(kernel_order_string).unwrap_unchecked(),
        )
        .unwrap_unchecked()
    };

    (input_tensor, kernels_tensor)
}

/// Initializes DLA by setting up necessary heap allocator from headsail-bsp. This should be called only once in the program.
#[no_mangle]
pub unsafe extern "C" fn dla_init() {
    // SAFETY: `init_heap` must be called once only
    unsafe { init_heap() };
}

/// Executes Conv2D on DLA with given parameters and writes result to output buffer.
#[no_mangle]
pub unsafe extern "C" fn dla_conv2d(
    input_data: *const i8,
    kernel_data: *const i8,
    output: *mut i8,
    input_channels: usize,
    input_height: usize,
    input_width: usize,
    input_order: *const c_char,
    kernel_amount: usize,
    kernel_channels: usize,
    kernel_height: usize,
    kernel_width: usize,
    kernel_order: *const c_char,
    pad_top: u32,
    pad_right: u32,
    pad_left: u32,
    pad_bottom: u32,
    pad_value: i32,
    stride_x: u32,
    stride_y: u32,
    mac_clip: u32,
    pp_clip: u32,
) {
    let (input_tensor, kernels_tensor) = unsafe {
        ffi_data_import(
            input_data,
            input_channels,
            input_height,
            input_width,
            input_order,
            kernel_data,
            kernel_amount,
            kernel_channels,
            kernel_height,
            kernel_width,
            kernel_order,
        )
    };

    let result: Tensor3<i8> = conv2d(
        input_tensor,
        kernels_tensor,
        Some(Padding {
            top: pad_top,
            right: pad_right,
            left: pad_left,
            bottom: pad_bottom,
            padding_value: pad_value,
        }),
        Some(Stride {
            x: stride_x,
            y: stride_y,
        }),
        Some(mac_clip),
        Some(pp_clip),
        None,
    );
    unsafe {
        core::ptr::copy_nonoverlapping(result.to_buffer().as_mut_ptr(), output, result.get_size())
    };
}

/// Executes Conv2D + ReLU on DLA with given parameters and writes result to output buffer.
#[no_mangle]
pub unsafe extern "C" fn dla_conv2d_relu(
    input_data: *const i8,
    kernel_data: *const i8,
    output: *mut i8,
    input_channels: usize,
    input_height: usize,
    input_width: usize,
    input_order: *const c_char,
    kernel_amount: usize,
    kernel_channels: usize,
    kernel_height: usize,
    kernel_width: usize,
    kernel_order: *const c_char,
    pad_top: u32,
    pad_right: u32,
    pad_left: u32,
    pad_bottom: u32,
    pad_value: i32,
    stride_x: u32,
    stride_y: u32,
    mac_clip: u32,
    pp_clip: u32,
) {
    let (input_tensor, kernels_tensor) = unsafe {
        ffi_data_import(
            input_data,
            input_channels,
            input_height,
            input_width,
            input_order,
            kernel_data,
            kernel_amount,
            kernel_channels,
            kernel_height,
            kernel_width,
            kernel_order,
        )
    };

    let result: Tensor3<i8> = conv2d_relu(
        input_tensor,
        kernels_tensor,
        Some(Padding {
            top: pad_top,
            right: pad_right,
            left: pad_left,
            bottom: pad_bottom,
            padding_value: pad_value,
        }),
        Some(Stride {
            x: stride_x,
            y: stride_y,
        }),
        Some(mac_clip),
        Some(pp_clip),
        None,
    );
    unsafe {
        core::ptr::copy_nonoverlapping(result.to_buffer().as_mut_ptr(), output, result.get_size())
    };
}

/// Executes Conv2D + Bias on DLA with given parameters and writes result to output buffer.
/// # Arguments
///
/// * `bias` - Bias is actually i16 in hardware, here we use 32 for TVM compatibility
#[no_mangle]
pub unsafe extern "C" fn dla_conv2d_bias(
    input_data: *const i8,
    kernel_data: *const i8,
    // NOTE:
    bias: *const i32,
    output: *mut i8,
    input_channels: usize,
    input_height: usize,
    input_width: usize,
    input_order: *const c_char,
    kernel_amount: usize,
    kernel_channels: usize,
    kernel_height: usize,
    kernel_width: usize,
    kernel_order: *const c_char,
    bias_length: usize,
    pad_top: u32,
    pad_right: u32,
    pad_left: u32,
    pad_bottom: u32,
    pad_value: i32,
    stride_x: u32,
    stride_y: u32,
    mac_clip: u32,
    pp_clip: u32,
) {
    let (input_tensor, kernels_tensor) = unsafe {
        ffi_data_import(
            input_data,
            input_channels,
            input_height,
            input_width,
            input_order,
            kernel_data,
            kernel_amount,
            kernel_channels,
            kernel_height,
            kernel_width,
            kernel_order,
        )
    };

    let bias: Vec<i16> = unsafe { slice::from_raw_parts(bias as *const i16, bias_length).to_vec() };

    let result = conv2d_bias(
        input_tensor,
        kernels_tensor,
        bias,
        Some(Padding {
            top: pad_top,
            right: pad_right,
            left: pad_left,
            bottom: pad_bottom,
            padding_value: pad_value,
        }),
        Some(Stride {
            x: stride_x,
            y: stride_y,
        }),
        Some(mac_clip),
        Some(pp_clip),
        None,
    );
    unsafe {
        core::ptr::copy_nonoverlapping(result.to_buffer().as_mut_ptr(), output, result.get_size())
    };
}

/// Executes Conv2D + Bias + ReLU on DLA with given parameters and writes result to output buffer.
///
/// # Arguments
///
/// * `bias` - Buffer containing bias data. NOTE: Bias is actually i16 in hardware, here we use 32 for TVM compatibility
#[no_mangle]
pub unsafe extern "C" fn dla_conv2d_bias_relu(
    input_data: *const i8,
    kernel_data: *const i8,
    bias: *const i32,
    output: *mut i8,
    input_channels: usize,
    input_height: usize,
    input_width: usize,
    input_order: *const c_char,
    kernel_amount: usize,
    kernel_channels: usize,
    kernel_height: usize,
    kernel_width: usize,
    kernel_order: *const c_char,
    bias_length: usize,
    pad_top: u32,
    pad_right: u32,
    pad_left: u32,
    pad_bottom: u32,
    pad_value: i32,
    stride_x: u32,
    stride_y: u32,
    mac_clip: u32,
    pp_clip: u32,
) {
    let (input_tensor, kernels_tensor) = unsafe {
        ffi_data_import(
            input_data,
            input_channels,
            input_height,
            input_width,
            input_order,
            kernel_data,
            kernel_amount,
            kernel_channels,
            kernel_height,
            kernel_width,
            kernel_order,
        )
    };

    let bias: Vec<i16> = unsafe {
        slice::from_raw_parts(bias as *const i32, bias_length)
            .into_iter()
            .map(|x| (*x).clamp(i16::MIN as i32, i16::MAX as i32) as i16)
            .collect()
    };

    let result = conv2d_bias_relu(
        input_tensor,
        kernels_tensor,
        bias,
        Some(Padding {
            top: pad_top,
            right: pad_right,
            left: pad_left,
            bottom: pad_bottom,
            padding_value: pad_value,
        }),
        Some(Stride {
            x: stride_x,
            y: stride_y,
        }),
        Some(mac_clip),
        Some(pp_clip),
        None,
    );

    let input_order_string = unsafe { CStr::from_ptr(input_order).to_str().unwrap_unchecked() };
    unsafe {
        core::ptr::copy_nonoverlapping(
            result
                .to_buffer_with_order(Order3::try_from(input_order_string).unwrap_unchecked())
                .as_mut_ptr(),
            output,
            result.get_size(),
        )
    };
}

/// # Arguments
///
/// * `bias` - Buffer containing bias data. NOTE: Bias is actually i16 in hardware, here we use 32 for TVM compatibility
#[no_mangle]
pub unsafe extern "C" fn dla_tvm_qnn_conv2d_bias(
    input_data: *const i8,
    kernel_data: *const i8,
    bias: *const i32,
    output: *mut i32,
    input_channels: usize,
    input_height: usize,
    input_width: usize,
    input_order: *const c_char,
    kernel_amount: usize,
    kernel_channels: usize,
    kernel_height: usize,
    kernel_width: usize,
    kernel_order: *const c_char,
    bias_length: usize,
    pad_top: u32,
    pad_right: u32,
    pad_left: u32,
    pad_bottom: u32,
    pad_value: i32,
    stride_x: u32,
    stride_y: u32,
    mac_clip: u32,
    pp_clip: u32,
) {
    let (input_tensor, kernels_tensor) = unsafe {
        ffi_data_import(
            input_data,
            input_channels,
            input_height,
            input_width,
            input_order,
            kernel_data,
            kernel_amount,
            kernel_channels,
            kernel_height,
            kernel_width,
            kernel_order,
        )
    };

    // NOTE:(20241025 vaino-waltteri.granat@tuni.fi) TVM expects 32-bit bias, but DLA only support 16-bit bias, so we clip the incoming bias
    // to range suitable for DLA
    let bias: Vec<i16> = unsafe {
        slice::from_raw_parts(bias as *const i32, bias_length)
            .into_iter()
            .map(|x| (*x).clamp(i16::MIN as i32, i16::MAX as i32) as i16)
            .collect()
    };


    let mut result: Tensor3<i8> = conv2d_bias(
        input_tensor,
        kernels_tensor,
        bias,
        Some(Padding {
            top: pad_top,
            right: pad_right,
            left: pad_left,
            bottom: pad_bottom,
            padding_value: 0,
        }),
        Some(Stride {
            x: stride_x,
            y: stride_y,
        }),
        Some(mac_clip),
        Some(pp_clip),
        None,
    );

    let input_order_string = unsafe { CStr::from_ptr(input_order).to_str().unwrap_unchecked() };

    // TVM requantization and clip
    // NOTE:(20240927 vaino-waltteri.granat@tuni.fi) on DLA clipping behaviour with TVM.
    // DLA's conv2d arithmetic is done at 16 bit width, but the output of the DLA is limited to 8 bits.
    // To comply with TVM's expected value range our solution is to bit shift/clip the 16-bit result of
    // conv2d by 8 bits and shift if back in the driver. This causes some amount of data loss due to
    // the lost granularity of the values. The clipping amount is set by the pp_clip argument.
    let mut res_i32: Vec<i32> = result.to_buffer_with_order(Order3::try_from(input_order_string).unwrap_unchecked())
                                       .iter().map(|x: &i8| (*x as f32 * u32::pow(2, pp_clip) as f32) as i32).collect();

    unsafe {
        core::ptr::copy_nonoverlapping(
            res_i32.as_mut_ptr(),
            output,
            result.get_size(),
        )
    };
}
