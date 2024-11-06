use alloc::vec::*;
use core::ffi::c_char;
use ndarray::{s, Array, Array3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Order3 {
    CHW,
    CWH,
    HWC,
    HCW,
    WHC,
    WCH,
}

impl Order3 {
    fn into_position(self) -> [usize; 3] {
        match self {
            Order3::CHW => [0, 1, 2],
            Order3::CWH => [0, 2, 1],
            Order3::HWC => [1, 2, 0],
            Order3::HCW => [1, 0, 2],
            Order3::WHC => [2, 1, 0],
            Order3::WCH => [2, 0, 1],
        }
    }
}

impl TryFrom<&str> for Order3 {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "CHW" => Ok(Order3::CHW),
            "CWH" => Ok(Order3::CWH),
            "HWC" => Ok(Order3::HWC),
            "HCW" => Ok(Order3::HCW),
            "WHC" => Ok(Order3::WHC),
            "WCH" => Ok(Order3::WCH),
            _ => Err(()),
        }
    }
}

impl TryFrom<[c_char; 3]> for Order3 {
    type Error = ();
    fn try_from(s: [c_char; 3]) -> Result<Self, Self::Error> {
        match s {
            // C = 0x43, H = 0x48, W = 0x57
            [0x43, 0x48, 0x57] => Ok(Order3::CHW),
            [0x43, 0x57, 0x48] => Ok(Order3::CWH),
            [0x48, 0x57, 0x43] => Ok(Order3::HWC),
            [0x48, 0x43, 0x57] => Ok(Order3::HCW),
            [0x57, 0x48, 0x43] => Ok(Order3::WHC),
            [0x57, 0x43, 0x48] => Ok(Order3::WCH),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tensor3<T> {
    data: Array3<T>,
    order: Order3,
}

impl<T: Clone> Tensor3<T> {
    // Creates a new Tensor3 with the specified dimensions, initial value, and order
    pub fn new(
        channels: usize,
        height: usize,
        width: usize,
        initial_value: T,
        order: Order3,
    ) -> Self {
        let data = Array::from_elem((channels, height, width), initial_value);
        Tensor3 { data, order }
    }

    pub fn channels(&self) -> usize {
        let dim_order: [usize; 3] = self.order.into_position();
        let position = unsafe { dim_order.iter().position(|&r| r == 0).unwrap_unchecked() };
        self.data.raw_dim()[position]
    }
    pub fn height(&self) -> usize {
        let dim_order: [usize; 3] = self.order.into_position();
        let position = unsafe { dim_order.iter().position(|&r| r == 1).unwrap_unchecked() };
        self.data.raw_dim()[position]
    }
    pub fn width(&self) -> usize {
        let dim_order: [usize; 3] = self.order.into_position();
        let position = unsafe { dim_order.iter().position(|&r| r == 2).unwrap_unchecked() };
        self.data.raw_dim()[position]
    }

    /// Creates a new Tensor3 from a data buffer with the specified order
    pub fn from_array3(data: Array3<T>, order: Order3) -> Self {
        Tensor3 { data, order }
    }

    /// Creates a new Tensor3 from a data buffer with the specified order
    pub fn from_data_buffer(
        channels: usize,
        height: usize,
        width: usize,
        data_buffer: Vec<T>,
        order: Order3,
    ) -> Result<Self, &'static str> {
        if data_buffer.len() != channels * height * width {
            return Err("Data buffer size does not match specified dimensions");
        }

        let standard_shape = [channels, height, width];
        let dim_order: [usize; 3] = order.into_position();
        let fst = standard_shape[dim_order[0]];
        let snd = standard_shape[dim_order[1]];
        let thd = standard_shape[dim_order[2]];

        let data = Array::from_shape_vec((fst, snd, thd), data_buffer)
            .map_err(|_| "Failed to create array from data buffer")?;

        Ok(Tensor3 { data, order })
    }

    /// Get the number of element in ndarray
    pub fn get_size(&self) -> usize {
        self.data.len()
    }

    /// Matches order field value to height, width and channels parameters
    fn get_dimension_order_values(&self, order: Option<Order3>) -> [usize; 3] {
        let mut out = [0; 3];

        // Use self value if no order was given
        let order: [usize; 3] = match order {
            Some(order) => order.into_position(),
            None => self.order.into_position(),
        };

        for (i, x) in order.into_iter().enumerate() {
            let param = match x {
                0 => self.channels(),
                1 => self.height(),
                2 => self.width(),
                _ => unimplemented!(),
            };
            out[i] = param;
        }
        out
    }

    /// Returns a reference to the element at the specified position
    pub fn get(&self, channel: usize, row: usize, col: usize) -> Option<&T> {
        self.data.get((channel, row, col))
    }

    /// Returns a mutable reference to the element at the specified position
    pub fn get_mut(&mut self, channel: usize, row: usize, col: usize) -> Option<&mut T> {
        self.data.get_mut((channel, row, col))
    }

    /// Sets the element at the specified position
    pub fn set(
        &mut self,
        channel: usize,
        row: usize,
        col: usize,
        value: T,
    ) -> Result<(), &'static str> {
        if let Some(elem) = self.data.get_mut((channel, row, col)) {
            *elem = value;
            Ok(())
        } else {
            Err("Index out of bounds")
        }
    }

    /// Returns the dimensions of the array
    pub fn dimensions(&self) -> (usize, usize, usize) {
        (self.channels(), self.height(), self.width())
    }

    /// Gets the current order of the array
    pub fn order(&self) -> Order3 {
        self.order
    }

    /// Sets a new order for the array
    pub fn permute(&mut self, order: Order3) {
        // Early return if already in order
        if self.order == order {
            return;
        }

        if self.order == Order3::CHW {
            // Transmute to target order
            let new_order: [usize; 3] = order.into_position();
            self.data = self.data.clone().permuted_axes(new_order);
            self.order = order;
            return;
        }

        // Transmute to standard order
        let std_order: [usize; 3] = self.order.into_position();
        let std = self.data.clone().permuted_axes(std_order);

        // Transmute to target order
        let new_order: [usize; 3] = order.into_position();
        self.data = std.permuted_axes(new_order);
        self.order = order;
    }

    /// Converts the internal buffer to CHW order
    fn to_chw_buffer(&self) -> Vec<T> {
        self.to_buffer_with_order(Order3::CHW)
    }

    /// Converts the 3D array to a linear buffer according to the current order
    pub fn to_buffer(&self) -> Vec<T> {
        let mut buffer = Vec::with_capacity(self.get_size());
        for x in Array::from_iter(self.data.iter().cloned()) {
            buffer.push(x)
        }
        buffer
    }

    /// Converts the 3D array to a linear buffer according to the specified order
    pub fn to_buffer_with_order(&self, order: Order3) -> Vec<T> {
        // If order is correct no need to permute
        if order == self.order {
            return self.to_buffer();
        }

        let mut data = self.clone();
        data.permute(order);
        data.to_buffer()
    }
}

pub fn rescale(
    tensor: &mut Tensor3<i8>,
    pre_scale: f32,
    input_zero: i32,
    output_zero: i32,
    input_scale: f32,
    output_scale: Vec<f32>,
) {
    // Ensure that the number of scaling factors matches the number of channels.
    assert_eq!(
        tensor.channels(),
        output_scale.len(),
        "Mismatch in number of channels"
    );

    // Iterate over each channel and apply the scaling factor.
    for (channel, scale) in output_scale.iter().enumerate() {
        let mut channel_slice = match tensor.order() {
            Order3::CHW | Order3::CWH => tensor.data.slice_mut(s![channel, .., ..]),
            Order3::HWC | Order3::WHC => tensor.data.slice_mut(s![.., .., channel]),
            Order3::HCW | Order3::WCH => tensor.data.slice_mut(s![.., channel, ..]),
        };

        channel_slice.map_inplace(|x| {
             let value = (input_scale / scale) * (*x as f32 * pre_scale - input_zero as f32)
                 + output_zero as f32;
            *x = value.clamp(i8::MIN as f32, i8::MAX as f32) as i8
        });
    }
}
