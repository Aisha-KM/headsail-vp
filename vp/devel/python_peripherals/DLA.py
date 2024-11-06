import math
import random
import os
import sys

NAME = "DLA"

# Memory banks
MEMORY_BANK_ADDR = 0x70000000
MEMORY_BANK_SIZE = 0x8000
NO_MEMORY_BANKS = 16

# Register map
DLA_ADDR = 0xFF700000
REG_BASE_ADDR = 0x1000
MEM_SIZE = 0x68
REG_WIDTH = 32

# Status register
STATUS_ADDR = 0x0 
BUF_DONE_OFFSET = 0
MAC_DONE_OFFSET = 1
PP_DONE_OFFSET = 2
DMA_IRQ_OFFSET = 3

# Control register
CTRL_ADDR = 0x4 
CPU_FE_OFFSET = 0
HP_RST_OFFSET = 4
SW_IRQ_OFFSET = 8

# Buffer control
BUF_CTRL = 0x8
CONV_MODE_OFFSET = 0
READ_A_VALID_OFFSET = 4
READ_B_VALID_OFFSET = 8

# Mac control
MAC_CTRL = 0xC
SIMD_SELECT_OFFSET= 1
MAC_CLIP_OFFSET = 8

# PP control
PP_CTRL = 0x10
ACTIVE_MODE_OFFSET = 0
RELU_OFFSET_UNUSED = 2
MAX_OFFSET_UNUSED = 4
PP_SELECT_OFFSET = 6
POOL_MODE_OFFSET_UNUSED = 7
ROUNDING_OFFSET = 9
CTRL_VLD_OFFSET_UNUSED = 10
PP_CLIP_OFFSET = 16

# Buffer input
BUF_INPUT = 0x14
BUF_WIDTH_OFFSET = 0
BUF_HEIGHT_OFFSET = 9
BUF_CHANNELS_OFFSET = 18


# Buffer kernel 0
BUF_KERNEL_0 = 0x18
BUF_KERNEL_0_WIDTH_OFFSET = 0
BUF_KERNEL_0_HEIGHT_OFFSET = 4
BUF_KERNEL_0_S_CHANNELS_OFFSET = 8

# Buffer kernel 1
BUF_KERNEL_1 = 0x1C
BUF_KERNEL_1_NUM_OFFSET = 0

# Buffer padding
BUF_PAD = 0x20
BUF_PAD_TOP_OFFSET = 0
BUF_PAD_RIGHT_OFFSET = 4
BUF_PAD_BOTTOM_OFFSET = 8
BUF_PAD_LEFT_OFFSET = 12
BUF_PAD_VALUE_OFFSET = 16

# Buffer stride
BUF_STRIDE = 0x24 
BUF_STRIDE_X_OFFSET = 0
BUF_STRIDE_Y_OFFSET = 16

# PP input
PP_INPUT = 0x28
PP_INPUT_WIDTH_OFFSET = 0
PP_INPUT_HEIGHT_OFFSET = 16

# Buffer data bank
BUF_DATA_BANK = 0x2C
BUF_DATA_BANK_A_OFFSET = 0
BUF_DATA_BANK_B_OFFSET = 16

# Buffer data wait A
BUF_DATA_WAIT_A = 0x30
BUF_DATA_WAIT_A_OFFSET = 0

# Buffer data wait B
BUF_DATA_WAIT_B = 0x34
BUF_DATA_WAIT_B_OFFSET = 0

# Buffer pipe stall
BUF_PIPE_STALL_STALL_CYCLES = 0x38
BUF_PIPE_STALL_STALL_CYCLES_OFFSET = 0

# Power control
POWER_CTRL = 0x4C
POWER_CTRL_DOWN_0_OFFSET = 0
POWER_CTRL_DOWN_1_OFFSET = 1
POWER_CTRL_DOWN_2_OFFSET = 2
POWER_CTRL_ISO_OFFSET = 3

# Power status
POWER_STAT = 0x50
POWER_STAT_ACK_0_OFFSET = 0
POWER_STAT_ACK_1_OFFSET = 1
POWER_STAT_ACK_2_OFFSET = 2

# DMA control
DMA_CTRL = 0x44
DMA_CTRL_READ_EVENT_OFFSET = 0
DMA_CTRL_WRITE_EVENT_OFFSET = 0

# DMA padding
DMA_PAD_CONFIG = 0x48
DMA_PAD_CONFIG_OFFSET = 0

# MAC_SAT_MAX
MAC_SAT_MAX = 0x54
MAC_SAT_MAX_OFFSET = 0

# MAC_SAT_MIN
MAC_SAT_MIN = 0x58
MAC_SAT_MIN_OFFSET = 0

# PP_AXI_WRITE
PP_AXI_WRITE = 0x5C
PP_AXI_WRITE_ADDRESS_OFFSET = 0

# PP_AXI_READ
PP_AXI_READ = 0x60
PP_AXI_READ_ADDRESS_OFFSET = 0

# Handshake
HANDSHAKE = 0x64
HANDSHAKE_BUFFER_VALID_OFFSET = 0
HANDSHAKE_MAC_VALID_OFFSET = 1
HANDSHAKE_POOL_VALID_OFFSET = 2
HANDSHAKE_ACTIVE_VALID_OFFSET = 3
HANDSHAKE_BUFFER_ENABLE_OFFSET = 4
HANDSHAKE_MAC_ENABLE_OFFSET =  5
HANDSHAKE_ACTIVE_ENABLE_OFFSET = 6
HANDSHAKE_POOL_ENABLE_OFFSET = 7
HANDSHAKE_BIAS_ENABLE_OFFSET = 8
HANDSHAKE_BYPASS_ENABLE_OFFSET = 9


# Utilities
def reshape_to_cwh(data):
    """Takes tensor in [height, width, channel] format and reshapes it to [channel, width, height]"""
    in_channels = len(data[0][0])
    in_width = len(data[0])
    in_heigth = len(data)

    # Initialize CWH array
    output = [[[0 for _ in range(in_heigth)] for _ in range(in_width)] for _ in range(in_channels)]

    # Reshape
    for ch in range(in_channels):
        for w in range(in_width):
            for h in range(in_heigth):
                output[ch][w][h] = data[h][w][ch]
    return output

def zeroes(shape):
    """Numpy style zeros functions. Take in shape as tuple and creates tensor of zeros by the dimensions defined in the tuple

    Params:
    shape -- Tuple(dimx, dimy, dimz...) defining dimensions of the resulting tensor

    Returns:
    tensor -- ndimensional tensor filled with zeros
    """
    innner_most_array = [0 for _ in range(shape[-1])]

    if len(shape) == 1:
        return innner_most_array

    for dim in reversed(shape[:-1]):
        array = [innner_most_array for _ in range(dim)]
        innner_most_array = array

    return array

def get_size(tensor):
    """Gets number of elements in ndimensional tensor

    Params:
    tensor -- tensor to count elements int

    Returns:
    count -- Int number of elements in input tensor
    """
    shape = get_shape(tensor)

    count = 1
    for dim in shape:
        count *= dim

    return count

def get_shape(tensor):
    """Get dimensionality of ndimensional tensor

    Params:
    tensor -- input tensor to find dimensionality from

    Returns:
    shape -- Tuple(dimx, dimy, dimz...) of the dimensionality of the input tensor

    """
    shape = []
    while isinstance(tensor, list):
        shape.append(len(tensor))
        tensor = tensor[0]
    return tuple(shape)

def flatten(tensor, order='C'):
    """Flattens ndimensional tensor to one dimensional tensor/vector

    Params:
    tensor -- ndimensional tensor to flatten
    order -- order of flattenning, C=Row-major order, F=Column-major order

    Returns:
    output -- 1-dimensional tensor
    """
    if order == 'F':
        tensor = transpose(tensor)
    elif order == 'C':
        tensor = tensor
    else:
        raise Exception("Invalid order for flattening: {}. Orders supported are C=Row-major, F=Column-major".format(order))

    output = []
    if isinstance(tensor[0], list):
        for l in tensor:
            output = output + (flatten(l))
        return output
    for x in tensor:
        output.append(x)
    return output

def transpose(tensor):
    """Swaps rows and column in a 2d matrix

    Params:
    tensor -- 2 dimensional matrix to perform transpose on

    Retuns:
    output -- transposed 2d tensor
    """
    return [list(x) for x in zip(*tensor)]

def reshape(tensor, shape):
    """Numpy style reshape. Reshapes input tensor to dimensionality defined by the shape parameter. Input tensor and shape have equal number of elements.

    Params:
    tensor -- ndimensional tensor to reshape
    shape -- Tuple(dimx, dimy, dimz...) defining the shape of the output tensor

    Returns:
    output -- ndimensional tensor shaped from input tensor with shape of shape parameter
    """

    def construct(flat, shape):
        if len(shape) == 1:
            return flat[:shape[0]]
        sub_size = int(len(flat) / shape[0])
        return [construct(flat[i * sub_size: (i + 1) * sub_size], shape[1:]) for i in range(shape[0])]

    output = zeroes(shape)
    assert get_size(output) == get_size(tensor), "Assert failed! Reshape incompatible dimensions"
    flat = flatten(tensor)

    return construct(flat, shape)


def cast_long_to_signed_byte(value):
    """Bitwise cast of unsigned char to signed char.

    Params:
    value -- Int Unsigned value to cast to signed char

    Returns:
    byte -- Int Signed value in range -128..127
    """
    assert(0 <= value <= 0xFF), "Assert failed! Value doesn't fit byte"
    value = value & 0xFF
    if value <= 127:
        return value
    return value - 256

def cast_long_to_signed_16(value):
    """Bitwise cast of unsigned char to signed char.

    Params:
    value -- Int Unsigned value to cast to signed char

    Returns:
    byte -- Int Signed value in range -128..127
    """
    assert(0 <= value <= 0xFFFF), "Assert failed! Value doesn't fit 2 bytes"
    value = value & 0xFFFF
    if value <= 32767:
        return value
    return value - 65536



def separate_channels(data):
    """Reformats data so that each channels is it's own 2D array

    Params:
    data -- [[[Int]]] Data in format CWH

    Return:
    channel_matrices
    """
    num_channels, width, height = len(data), len(data[0]), len(data[0][0])
    channel_matrices = []
    for i in range(num_channels):
        channel_matrix = []
        for j in range(width):
            row = []
            for k in range(height):
                row.append(data[i][j][k])
            channel_matrix.append(row)
        channel_matrices.append(channel_matrix)
    return channel_matrices

def bit_not(n, numbits=32):
    """Reverses bits in a Python integer/long

    Params:
    n -- number to reverse
    numbits -- bit width of the number

    Returns:
    !n -- n with bits reversed
    """
    return (1 << numbits) - 1 - n

def print_matrix(A, name="", pformat="decimal"):
    """Print matrix"""
    print(name)

    if not isinstance(A[0], list):
        if pformat=="decimal":
            row = " ".join("{:4}".format(value) for value in A)
        elif pformat=="hexadecimal":
            row = " ".join("{:4}".format(hex(value & 0xff)[2:-1]) for value in A)
        else:
            row = "invalid pformat {}".format(pformat)
        print(row)
        return

    for x in range(len(A)):
        if pformat=="decimal":
            row = " ".join("{:4}".format(value) for value in A[x])
        elif pformat=="hexadecimal":
            row = " ".join("{:4}".format(hex(value & 0xff)[2:-1]) for value in A[x])
        else:
            row = "invalid pformat {}".format(pformat)
        print(row)

def memory_bank_to_offset(bank):
    """ Converts memory bank index to memory offset

    Params:
    bank -- Bank index

    Returns:
    offset -- offset of bank with given index
    """
    return bank * MEMORY_BANK_SIZE

def execute_for_all_elements(f, tensor, *args):
    """Execute function that takes a single matrix element for all element in a matrix.

    Params:
    f -- Function(Int, *args)->Int Function to be applied to matrix elements. Needs to return same element as it operates on.
    tensor -- ndarray(Int) Multidimensional array with number elements to apply function f to.
    *args -- *args Additional arguments needed by function f.

    Returns:
    tensor -- ndarray(Int) Tensor with function f applied to all it's elements
    """
    if isinstance(tensor, list):  # Check if tensor is a list
        return [execute_for_all_elements(f, x, *args) for x in tensor]
    else:  # Base case: tensor is not a list, apply f
        return f(tensor, *args)

def clip(value, clip_amount, bit_width=8):
    """Value to possibly clip is clipped to max of bit length set by clip
    params:
    value = value to clip
    clip =  number of bits to clip
    bit_width = bit width of the given value
    return:
    clipped_value = value resulting from the clipping
    """
    upper_bound = pow(2, bit_width) // 2 - 1 # 127
    lower_bound = -upper_bound - 1
    clipped_value = value >> clip_amount
    if clipped_value > upper_bound:
        return upper_bound
    elif clipped_value < lower_bound:
        return lower_bound
    return clipped_value

def clip_value_to_i16(value):
    upper_bound = pow(2, 16) // 2 - 1
    lower_bound = -upper_bound - 1
    if value > upper_bound:
        return upper_bound
    elif value < lower_bound:
        return lower_bound
    return value


def rounding(value):
    """Round given value

    Params:
    value -- Float value to round

    Return:
    rounded -- Int Rounded value
    """
    if value > 127:
        return 127
    elif value < -128:
        return -128
    else:
        return value

class Padding:
    def __init__(self, left, right, top, bottom):
        self.left = left
        self.right = right
        self.top = top
        self.bottom = bottom

    def __str__(self):
        return "(L:{}, R:{}, T:{}, B:{})".format(self.left, self.right, self.top, self.bottom)

    def __repr__(self):
        return "(L:{}, R:{}, T:{}, B:{})".format(self.left, self.right, self.top, self.bottom)



# DLA
class MemoryBank:
    """Implements DLA memory bank"""
    def __init__(self, size):
        self.size = size
        self.clear_bank()

    def clear_bank(self):
        """Reset all values in the bank to 0."""
        self.mem = [0 for x in range(self.size)] # Initialize bank

    def write_buffer(self, offset, data):
        """Write data buffer to bank. Starting from given offset

        Params:
        offset -- starting address of the write
        data -- [Int] data to write to bank

        Returns:
        unwritten -- [Int] data that didn't fit to bank
        """
        for i in range(len(data)):
            if offset + i > self.size:
                return data[i:]
            self.mem[offset + i] = data[i]
        return []

    def write(self, offset, data):
        """Write data to given offset in bank.

        Params:
        data -- [Int] data to write to bank
        """
        assert(offset < self.size), "Assert failed! Offset overflow on bank write"
        self.mem[offset] = data


    def read(self, offset):
        """Read byte from memory at given offset"""
        return self.mem[offset]

class Dla:
    """Implements control flow and MMIO registers of DLA. This should be the top level component."""
    def __init__(self):
        self.mem = bytearray(MEM_SIZE) # Memory initalizaed
        self.mac = DlaMac()
        # Initialize memory banks
        self.banks = [MemoryBank(MEMORY_BANK_SIZE) for x in range(0, NO_MEMORY_BANKS)]

        # Initialize register to correct values
        # Start processing data in buffers
        self.process()

    def write(self, offset, value):
        """Writes value to register with given offset

        Params:
        offset -- Int memory address offset to write
        value -- Int value to write
        """
        self.mem[offset] = value

    def read(self, offset):
        """Read value of register with given offset

        Params:
        offset -- Int memory address offset to read

        Returns:
        value -- value of register in given offset
        """
        return self.mem[offset]

    def set_register(self, register, offset, width, value, preserve_register=True):
        """Set value of specific registers offset

        Params:
        register -- base address of 32-bit "register"
        offset -- integer representing bit offset to start modification from
        width -- width of the area in memory modified starting from offset
        value -- value to be written to register
        """
        if preserve_register:
            full_reg = self.mem[register:register+4]
            full_reg = full_reg[0] +  (full_reg[1] << 8)  + (full_reg[2] << 16) + (full_reg[3] << 24)
        else:
            full_reg = 0
        zeroing_mask = bit_not((pow(2,width)-1) << offset)

        full_reg = full_reg & zeroing_mask # Zero region to be written
        full_reg = full_reg | (value << offset)

        self.mem[register] = full_reg & 0xFF
        self.mem[register + 1] = (full_reg >> 8) & 0xFF
        self.mem[register + 2] = (full_reg >> 16) & 0xFF
        self.mem[register + 3] = (full_reg >> 24) & 0xFF

    def get_register(self, register, offset, width):
        """Read value of specific bits from a register

        Params:
        register -- integer of register to read from
        offset -- integer representing bit offset to start reading from
        width -- width of the area in memory to read starting from offset

        Returns:
        value -- value read from the specified bytes in the register
        """
        full_reg = self.mem[register:register+4]
        full_reg = full_reg[0] +  (full_reg[1] << 8)  + (full_reg[2] << 16) + (full_reg[3] << 24)
        value = full_reg >> offset
        mask = pow(2,width) - 1
        return value & mask

    def get_registers(self):
        """Get all registers

        Return:
        mem -- [Int] all registers values in DLA
        """
        return self.mem

    def print_register(self, register):
        """Print specific register in b format

        Params:
        register: integer representing register to print
        """
        print("{0:x}: {1:08b}|{2:08b}|{3:08b}|{4:08b}".format(register,
                                                              self.mem[register + 3],
                                                              self.mem[register + 2],
                                                              self.mem[register + 1],
                                                              self.mem[register]))

    def get_simd_mask(self):
        """Reads current simd mode and returns corresponding bitmask

        Returns:
        bitmask -- bitmask corresponding to the current simd mode
        """
        simd = self.get_register(MAC_CTRL, SIMD_SELECT_OFFSET, 2)
        if simd == 0:
            return 0xFF
        elif simd == 1:
            return 0xF
        elif simd == 2:
            return 0x3
        else:
            return 0xFF

    def print_registers(self):
        """Print all registers"""
        for x in range(0,HANDSHAKE+4, 4):
            self.print_register(x)
        return

    def get_memory_banks(self):
        """Get all memory banks

        Return:
        banks -- [MemoryBank] List of all the MemoryBank objects
        """
        return self.banks

    def handle_bank_write(self, request):
        """Writes data request to memory bank based on request.absolute address

        Params:
        request -- Renode request object
        """
        target_bank = (request.absolute - MEMORY_BANK_ADDR) // MEMORY_BANK_SIZE
        offset = request.offset - memory_bank_to_offset(target_bank)
        for byte in range(request.length):
            value = (request.value >> (byte * 8))
            value = cast_long_to_signed_byte(value) # NOTE: renode always uses unsigned bytes, so here need to cast u8 to i8
            self.banks[target_bank].write(offset + byte, value)

    def write_bank(self, bank, data):
        """Writes data to bank by index

        Params:
        bank -- index of bank to write to
        data -- [Int] data buffer to write to bank
        """
        self.banks[bank].write_buffer(0, data)

    def handle_bank_read(self, request):
        """Handle renode request of writing to bank address space

        Params:
        request -- renode request object

        Returns:
        value -- 32-bit clipped value of renode request
        """
        # NOTE: Renode can't handle over 32bit reads so 128-bit alignment isn't strictly enforced
        target_bank = (request.absolute - MEMORY_BANK_ADDR) // MEMORY_BANK_SIZE
        offset = request.absolute - MEMORY_BANK_ADDR - memory_bank_to_offset(target_bank)
        value = 0
        for i in range(0,4):
            value += (self.banks[target_bank].read(offset + i)) << (i * 8)
        return value & 0xFFFFFFFF # Clip to 32-bits

    def set_weight_data(self, data):
        """Sets weight data to memory banks

        Params:
        data -- [Int] Weight data in CHW format
        """

        bank_idx = self.get_register(BUF_DATA_BANK, BUF_DATA_BANK_A_OFFSET, 4)
        bank = self.banks[bank_idx]

        bytes_written = 0
        offset = 0
        while bytes_written < len(data):
            if offset > bank.size:
                bank_idx = bank_idx + 1
                assert(bank_idx < len(self.banks)), "Assert failed! Bank indexing overflow"
                bank = self.banks[bank_idx]
                offset = 0
            bank.write(offset, data[bytes_written])
            bytes_written += 1
            offset += 1

    def get_output_addr(self):
        """Get addr for outputting data set in PP_AXI_WRITE register

        Returns:
        addr -- Int addr for output
        """
        return self.get_register(PP_AXI_WRITE, PP_AXI_WRITE_ADDRESS_OFFSET, 32) & 0xFFFFFFFF

    def write_output(self, data, bit_width=8):
        """Writes output to arbitrary memory address

        Params:
        data -- [[[Int]]] data to write
        data_wisth -- Int width of data being written
        """

        # Read wanted output format and flatten
        filter_amount, height, width = get_shape(data)
        data = flatten(data)
        if data == []:
            print("WARN: output was empty.")
            return

        # Transform to 1d column-wise order while keeping the filters separate
        column_wise = []
        for h in range(height):
            for w in range(width):
                for f in range(filter_amount):
                    idx = (f * height * width) + w + (h * width)

                    if idx >= len(data):
                        print("Too big index:", idx)
                    else:
                        column_wise.append(data[idx])

        data = column_wise

        addr = self.get_output_addr()
        print("Writing output to:{:x} with bit width{}".format(addr, bit_width))

        # Allow writing to memory space of data banks
        if MEMORY_BANK_ADDR <= addr and addr < (MEMORY_BANK_ADDR + (NO_MEMORY_BANKS * MEMORY_BANK_SIZE)):
            bank_idx = (addr - MEMORY_BANK_ADDR) // MEMORY_BANK_SIZE
            bank = self.banks[bank_idx]

            values_written = 0
            offset = 0
            while values_written < len(data): # Data packing

                # Bank switching when current bank is filled
                if offset + (bit_width / 8) - 1 > bank.size:
                    bank_idx = bank_idx + 1
                    assert(bank_idx < len(self.banks)), "Assert failed!, Bank indexing overflow"
                    bank = self.banks[bank_idx]
                    offset = 0

                # Write chunk of data with correct simd width
                if bit_width == 32:
                    byte_3 = (data[values_written] & 0xFF000000) >> 24
                    byte_2 = (data[values_written] & 0x00FF0000) >> 16
                    byte_1 = (data[values_written] & 0x0000FF00) >> 8
                    byte_0 = data[values_written] & 0x000000FF
                    bank.write(offset, byte_3)
                    bank.write(offset + 1, byte_2)
                    bank.write(offset + 2, byte_1)
                    bank.write(offset + 3, byte_0)
                    values_written += 1
                    offset += 4
                elif bit_width == 16:
                    upper = (data[values_written] & 0xFF00) >> 8
                    lower = data[values_written] & 0x00FF
                    bank.write(offset, upper)
                    bank.write(offset + 1, lower)
                    values_written += 1
                    offset += 2
                elif bit_width == 8:
                    bank.write(offset, data[values_written] & 0xFF)
                    offset += 1
                    values_written += 1
                elif bit_width == 4:
                    fst = (data[values_written] & 0xF) << 4
                    snd = (data[values_written + 1] & 0xF)
                    combined = fst + snd
                    bank.write(offset, combined)
                    offset += 1
                    values_written += 2
                elif bit_width == 2:
                    fst = (data[values_written] & 0x3) << 6
                    snd = (data[values_written + 1] & 0x3) << 4
                    thrd = (data[values_written + 2] & 0x3) << 2
                    frth =  (data[values_written + 3] & 0x3)
                    combined = fst + snd + thrd + frth
                    bank.write(offset, combined)
                    offset += 1
                    values_written += 4
        else:
            print("WARNING: output written outside VP memory region")


    def set_input_data(self, data):
        """Sets input data to memory banks

        Params:
        data -- [Int] Input data in CHW format
        """
        bank_idx = self.get_register(BUF_DATA_BANK, BUF_DATA_BANK_B_OFFSET, 4)
        bank = self.banks[bank_idx]

        bytes_written = 0
        offset = 0
        while bytes_written < len(data):
            if offset > bank.size:
                bank_idx = bank_idx + 1
                assert(bank_idx < len(self.banks)), "Assert failed!, Bank indexing overflow"
                bank = self.banks[bank_idx]
                offset = 0
            bank.write(offset, data[bytes_written])
            bytes_written += 1
            offset += 1

    def get_weight_data(self):
        """Get all kernel/weight data from memory banks in FCWH format. (filter, image channel, width, height)

       Returns:
        filter_amount -- Int Number of filters
        width -- Int Width of input
        height -- Int Height of input
        data -- [Int] List of all the weight values in FCWH format
        """
        width = self.get_register(BUF_KERNEL_0, BUF_KERNEL_0_WIDTH_OFFSET, 4) + 1
        height = self.get_register(BUF_KERNEL_0, BUF_KERNEL_0_HEIGHT_OFFSET, 4) + 1
        s_channels = self.get_register(BUF_KERNEL_0, BUF_KERNEL_0_S_CHANNELS_OFFSET, 4) + 1
        filter_amount = self.get_register(BUF_KERNEL_1, BUF_KERNEL_1_NUM_OFFSET, 12) + 1
        input_channels = self.get_register(BUF_INPUT, BUF_CHANNELS_OFFSET, 12) + 1
        bank_idx = self.get_register(BUF_DATA_BANK, BUF_DATA_BANK_A_OFFSET, 4)
        bank = self.banks[bank_idx]

        offset = 0;
        data = []
        chunk = []
        while len(data) + len(chunk) < (filter_amount * input_channels * width * height):
            # Move to next bank
            if offset >= bank.size:
                bank_idx = bank_idx + 1
                bank = self.banks[bank_idx]
                offset = 0

            chunk.append(bank.read(offset))

            # Chunk reversing
            if len(chunk) == 8:
                data = data + chunk[::-1]
                chunk = []

            offset += 1

        # Append rest
        remaining = (filter_amount * input_channels * width * height) - len(data)
        data = data + chunk[remaining::-1][:remaining]

        # Column wise matrix formation
        column_wise = []

        for f in range(filter_amount):
            for c in range(input_channels):
                for h in range(height):
                    for w in range(width):
                        idx = c + (f * input_channels) + (input_channels*filter_amount) * w + (input_channels * filter_amount * width) * h
                        column_wise.append(data[idx])

        column_wise = reshape(column_wise, (filter_amount, input_channels, height, width))
        return filter_amount, s_channels, width, height, column_wise

    def get_input_data(self):
        # TODO: Only read as much data as is needed to fill input layer (C* W * H)
        """Get all input data from memory banks in CWH format

        Returns:
        channels -- Int Number of channels
        width -- Int Width of input
        Height -- Int Height of input
        data -- [[Int]] List of all the input values in CWH format
        """
        width = self.get_register(BUF_INPUT, BUF_WIDTH_OFFSET, 9) + 1
        height = self.get_register(BUF_INPUT, BUF_HEIGHT_OFFSET, 9) + 1
        channels = self.get_register(BUF_INPUT, BUF_CHANNELS_OFFSET, 12) + 1
        bank_idx = self.get_register(BUF_DATA_BANK, BUF_DATA_BANK_B_OFFSET, 4)
        bank = self.banks[bank_idx]

        print("channels:", channels, "width:", width, "height:", height)

        offset = 0;
        data = []
        chunk = []
        while len(data) + len(chunk) < (channels * width * height):
            # Move to next bank
            if offset >= bank.size:
                bank_idx = bank_idx + 1
                bank = self.banks[bank_idx]
                offset = 0
            chunk.append(bank.read(offset))

            # Chunk reversing
            if len(chunk) == 8:
                data = data + chunk[::-1]
                chunk = []

            offset += 1

        # Append rest
        remaining = (channels * width * height) - len(data)
        data = data + chunk[remaining::-1][:remaining]

        # Column wise matrix formation
        column_wise = []
        for c in range(channels):
            for h in range(height):
                for w in range(width):
                    idx = c + channels * w + (channels * width) * h
                    column_wise.append(data[idx])

        column_wise = reshape(column_wise, (channels, height, width))
        print_matrix(column_wise[0], "flat input:", pformat="decimal")
        return channels, width, height, column_wise

    def get_bias(self, values_to_read):
        """Get bias values from memory. Due to VP limitations bias needs to be in DLA memory banks.

        Params:
        values_to_read -- Int number of values in Bias FIFO

        Returns:
        bias -- [Int] Bias FIFO read from memory with 16 bit width
        """
        bias_addr = self.get_register(PP_AXI_READ, PP_AXI_READ_ADDRESS_OFFSET, 32)
        # NOTE:(20240626 vaino-waltteri.granat@tuni.fi) In VP bias needs to be written into the memory banks
        if MEMORY_BANK_ADDR <= bias_addr and bias_addr < (MEMORY_BANK_ADDR + (NO_MEMORY_BANKS * MEMORY_BANK_SIZE)):
            bias = []
            bank_idx = (bias_addr - MEMORY_BANK_ADDR) // MEMORY_BANK_SIZE
            bank = self.banks[bank_idx]
            offset = 0

            while len(bias) < values_to_read:
                # Bank switching when current bank is filled
                if offset > bank.size:
                    bank_idx = bank_idx + 1
                    assert(bank_idx < len(self.banks)), "Assert failed! Bank indexing overflow"
                    bank = self.banks[bank_idx]
                    offset = 0
                low_byte = bank.read(offset) & 0xFF
                high_byte = bank.read(offset + 1) & 0xFF
                value = (high_byte << 8) + low_byte
                bias.append(cast_long_to_signed_16(value))
                offset += 2 # 16 bit width
            return bias
        else:
            print("WARNING: trying to read bias outside of VP memory region as address: {:x}".format(bias_addr))
            return []

    def get_conv_params(self):
       """Get parameters for conv2d

       Returns:
       padding -- Padding object
       dilation -- Tuple (Int, Int) sets dilation in (x,y) direction
       stride -- Tuple (Int, Int) sets stride in (x,y) direction
       """
       pad_left = self.get_register(BUF_PAD, BUF_PAD_LEFT_OFFSET, 4)
       pad_right = self.get_register(BUF_PAD, BUF_PAD_RIGHT_OFFSET, 4)
       pad_top = self.get_register(BUF_PAD, BUF_PAD_TOP_OFFSET, 4)
       pad_bottom = self.get_register(BUF_PAD, BUF_PAD_BOTTOM_OFFSET, 4)
       stride_x = self.get_register(BUF_STRIDE, BUF_STRIDE_X_OFFSET, 4) + 1
       stride_y = self.get_register(BUF_STRIDE, BUF_STRIDE_Y_OFFSET, 4) + 1
       dilation_x = 1 # NOTE: Headsail's DLA doesn't support other dilations
       dilation_y = 1

       padding = Padding(pad_left, pad_right, pad_top, pad_bottom)

       return padding, (dilation_x, dilation_y), (stride_x, stride_y)

    def round(self, values):
        """Round values if register is set"""
        if self.get_register(PP_CTRL, ROUNDING_OFFSET, 1) == 1:
            return execute_for_all_elements(rounding, values)
        return values

    def pp_clip(self, values):
        """Clip pp values if register is set"""
        clip_amount = self.get_register(PP_CTRL, PP_CLIP_OFFSET, 5)
        if clip_amount > 0:
            return execute_for_all_elements(clip, values, clip_amount, 16)
        return values

    def mac_clip(self, values):
        """Clip mac values if register is set"""
        clip_amount = self.get_register(MAC_CTRL, MAC_CLIP_OFFSET, 5)
        if clip_amount > 0:
            return execute_for_all_elements(clip, values, clip_amount, 16)
        return values

    def handle_handshake(self):
        """Resets handshake registers correctly after succesful calculation"""
        # Buffer
        if self.get_register(HANDSHAKE, HANDSHAKE_BUFFER_ENABLE_OFFSET, 1) == 0:
            if self.get_register(HANDSHAKE, HANDSHAKE_BUFFER_VALID_OFFSET, 1) == 1:
                self.set_register(HANDSHAKE, HANDSHAKE_BUFFER_VALID_OFFSET, 1, 0)
                self.set_register(STATUS_ADDR, BUF_DONE_OFFSET, 1, 0)

        # Mac
        if self.get_register(HANDSHAKE, HANDSHAKE_MAC_ENABLE_OFFSET, 1) == 0:
            if self.get_register(HANDSHAKE, HANDSHAKE_MAC_VALID_OFFSET, 1) == 1:
                self.set_register(HANDSHAKE, HANDSHAKE_MAC_VALID_OFFSET, 1, 0)
                self.set_register(STATUS_ADDR, MAC_DONE_OFFSET, 1, 0)

        # Post Processor
        if self.get_register(HANDSHAKE, HANDSHAKE_BYPASS_ENABLE_OFFSET, 1) == 0:
            if self.get_register(HANDSHAKE, HANDSHAKE_ACTIVE_VALID_OFFSET, 1) == 1:
                self.set_register(HANDSHAKE, HANDSHAKE_ACTIVE_VALID_OFFSET, 1, 0)
                self.set_register(STATUS_ADDR, PP_DONE_OFFSET, 1, 0)

        # PROCESSJUMPTAG
    def process(self):
        """Runs next tick of the DLA state"""

        # After completion handle handshakes
        self.handle_handshake()

        # Don't move if done hasn't been acknowledged VP only
        if self.get_register(STATUS_ADDR, BUF_DONE_OFFSET, 1) or \
                self.get_register(STATUS_ADDR, MAC_DONE_OFFSET, 1) or \
                self.get_register(STATUS_ADDR, PP_DONE_OFFSET, 1):
            self.print_register(STATUS_ADDR)
            print("Status not cleared")
            return

        # Check if data is ready
        if not self.get_register(BUF_CTRL, READ_A_VALID_OFFSET, 1) or not self.get_register(BUF_CTRL, READ_B_VALID_OFFSET, 1):
            return

        # Load data from memory banks and reshape
        input_ch, input_w, input_h, input_data = self.get_input_data()

        kernel_amount, s_channels, kernel_w, kernel_h, kernel_data = self.get_weight_data()

        # Convonlution
        padding, dilation, stride = self.get_conv_params()

        print("input:", input_ch, input_w, input_h)
        print("kernel:", kernel_amount, s_channels, kernel_w, kernel_h)
        print("padding:", padding)
        print("dilation:", dilation)
        print("stride:", stride)
        print("CONV2D")


        # Pack output according to clipping
        output_bit_width = self.get_register(MAC_CTRL, MAC_CLIP_OFFSET, 5) if self.get_register(MAC_CTRL, MAC_CLIP_OFFSET, 5) > 0 else 32

        if self.get_register(HANDSHAKE, HANDSHAKE_MAC_ENABLE_OFFSET, 1):
            for i, r in enumerate(input_data):
                print_matrix(input_data[i], "{} INPUT:".format(i))

            print("Mac not enabled")
            # TODO: This might be not correct, make sure S_CHANNELS work like this
            padding_value = cast_long_to_signed_byte(self.get_register(BUF_PAD, BUF_PAD_VALUE_OFFSET, 8))
            res = self.mac.conv2d(input_data, kernel_data, padding, dilation, stride, padding_value=padding_value)

            # Clip results
            res = dla.mac_clip(res)
            for i, r in enumerate(res):
                print_matrix(res[i], "{} MAC:".format(i))


        if self.get_register(HANDSHAKE, HANDSHAKE_BYPASS_ENABLE_OFFSET, 1):
            if self.get_register(HANDSHAKE, HANDSHAKE_BIAS_ENABLE_OFFSET, 1):

                bias = self.get_bias(get_shape(res)[0]) # Bias needs to be applied to each layer coming out of the MAC
                print("BIAS:", bias)
                tmp = []
                for (i, r) in enumerate(res):
                    def curry_bias(bias):
                        def next(element):
                            return bias + element
                        return next

                    a = execute_for_all_elements(curry_bias(bias[i]), r)
                    tmp.append(a)

                res = tmp
                for (i, r) in enumerate(res):
                    print_matrix(r, "{} BIAS:".format(i))

            # ReLU (active low)
            if self.get_register(HANDSHAKE, HANDSHAKE_ACTIVE_ENABLE_OFFSET, 1):
                print("RELU")
                res = execute_for_all_elements(self.mac.relu_native, res)

        # Prevent overflowing i16 range
        if output_bit_width == 32:
            self.write_output(res, 32)
        else:
            res = execute_for_all_elements(clip_value_to_i16, res)
            # Clipping and rounding
            res = dla.pp_clip(res)
            res = execute_for_all_elements(rounding, res)
            for (i, r) in enumerate(res):
                print_matrix(r, "{} PP:".format(i))

            self.write_output(res, 8)



        # After calculating one layer the device needs new configuration
        # Set Done status
        self.set_register(STATUS_ADDR, BUF_DONE_OFFSET, 1, 1)
        self.set_register(STATUS_ADDR, MAC_DONE_OFFSET, 1, 1)
        self.set_register(STATUS_ADDR, PP_DONE_OFFSET, 1, 1)

        # Set data not ready
        self.set_register(BUF_CTRL, READ_A_VALID_OFFSET, 1, 0)
        self.set_register(BUF_CTRL, READ_B_VALID_OFFSET, 1, 0)


class DlaMac:
    """Implement DLA's MAC array operations Conv2d, Bias and ReLU"""
    def __init__(self):
        self.name = "DLA MAC"

    def conv2d_check_parameters(self, A, kernel, padding, dilation, stride):
        """Calculates outputs of convolution matrix based on preferred inputs

        Params:
        A -- Matrix in form [[Int]] to be convolved
        kernel -- Matrix in form of [[Int]] to use ase convolution kernel
        padding -- Padding(left, right, top, bottom) object
        dilation -- Tuple (Int, Int) sets dilation in (x,y) direction
        stride -- Tuple (Int, Int) sets stride in (x,y) direction

        Returns:
        A -- Matrix in form [[Int]] to be convolved
        kernel -- Matrix in form of [[Int]] to use ase convolution kernel
        w_out -- Width of the conv2d result matrix
        h_out -- Height of the conv2d result matrix
        w_middle_zero -- Bool signifying if conv2d has uneven width
        h_middle_zero -- Bool signifying if conv2d has uneven height
        """

        h_in, w_in = get_shape(A)
        h_kernel, w_kernel = get_shape(kernel)
        w_out = math.floor((w_in + padding.left + padding.right - (dilation[0] * (w_kernel - 1)) -1) / stride[0] +1)
        h_out = math.floor((h_in + padding.top + padding.bottom - (dilation[1] * (h_kernel - 1)) -1) / stride[1] +1)

        w_middle_zero = w_kernel % 2 == 1
        h_middle_zero = h_kernel % 2 == 1

        return int(w_out), int(h_out), w_middle_zero, h_middle_zero

    def pad_matrix(self, mat_in, padding, padding_value=0):

        # Add top padding
        for _ in range(padding.top):
            mat_in.insert(0, [padding_value] * len(mat_in[0]))

        # Add bottom padding
        for _ in range(padding.bottom):
            mat_in.append([padding_value] * len(mat_in[0]))

        # Add left and right padding to each row
        for i in range(len(mat_in)):
            mat_in[i] = [padding_value] * padding.left + mat_in[i] + [padding_value] * padding.right

        return mat_in


    def conv2d(self, input_img, kernels, padding, dilation=(1,1), stride=(1,1), padding_value=0):
        # Find output size of single produced filter
        # Number of output filters is defined by the number of kernels
        # Each inputed kernel is applied for the whole input entry
        w_out, h_out, w_middle_zero, h_middle_zero = self.conv2d_check_parameters(input_img[0], kernels[0][0], padding, dilation, stride)
        print("Kernels shape:", get_shape(kernels))
        print("Input shape:", get_shape(input_img))

        # Initialize output filters
        output_filters = [[[ 0 for _ in range(w_out)] for _ in range(h_out) ] for _ in range(len(kernels)) ] # np.zeros(kernel_out, w_out, h_out)
        print("Output shape:", get_shape(output_filters))

        h_kernel_max_offset = len(kernels[0][0]) // 2 # Kernel height max offset
        w_kernel_max_offset = len(kernels[0][0][0]) // 2 # Kernel width max offset

        # Pad inputs
        padded_input = []
        for channel in input_img:
            padded_input.append(self.pad_matrix(channel, padding, padding_value=padding_value))

        # Apply each kernel to input_img
        for (kernel_idx, kernel) in enumerate(kernels):
            if w_middle_zero:
                center_x_0 = w_kernel_max_offset * dilation[0]
            else:
                center_x_0 = w_kernel_max_offset * dilation[0]

            if h_middle_zero:
                center_y_0 = h_kernel_max_offset * dilation[1]
            else:
                center_y_0 = h_kernel_max_offset * dilation[1]

            for h in range(h_out):
                if h_middle_zero:
                    center_y = center_y_0 + (h * stride[1])
                    range_y = [center_y + k * dilation[1] for k in range(-h_kernel_max_offset, h_kernel_max_offset + 1)]
                else:
                    center_y = center_y_0 + (h * stride[1]) # Calculate from top left of center
                    range_y = [center_y + k * dilation[1] for k in range(-h_kernel_max_offset, h_kernel_max_offset)]

                for w in range(w_out):
                    if w_middle_zero:
                        center_x = center_x_0 + (w * stride[0])
                        range_x = [center_x + k * dilation[0] for k in range(-w_kernel_max_offset, w_kernel_max_offset + 1)]
                    else:
                        center_x = center_x_0 + (w * stride[0])
                        range_x = [center_x + k * dilation[0] for k in range(-w_kernel_max_offset, w_kernel_max_offset)]

                    # Sum each channel with current kernel
                    channel_sum = 0
                    for (channel_idx, channel_data) in enumerate(padded_input):

                        # Constuct a sub matrix
                        # NOTE: Do not use zeroes here, for some reason IronPython can't correctly index nested lists produced by a recursive function. (20240527 vaino-waltteri.granat@tuni.fi)
                        mat_sub = [[0 for _ in range_x ] for _ in range_y ] # np.zeros(w_out, h_out)

                        for mat_y in range(len(range_y)):
                            for mat_x in range(len(range_x)):
                                mat_sub[mat_y][mat_x] = channel_data[range_y[mat_y]][range_x[mat_x]]

                        channel_res = self.mat_sum(self.matmul_element_wise(mat_sub, kernel[channel_idx]))
                        channel_sum += channel_res

                    output_filters[kernel_idx][h][w] = channel_sum

        return output_filters

    # ReLU
    def relu_native(self, x):
        """Perform ReLU for input,

        Params:
        x -- constant input Int

        Returns:
        ReLU(x) -- always positive integer with ReLU applied
        """
        if x > 0:
            return x
        else:
            return 0

    # Bias
    def bias_native(self, x, b):
        """Performs bias operation for input
        Params:
        x -- constant input Int
        b -- constant input bias Int

        Returns:
        sum -- x + b
        """
        return x + b

    def matsum_element_wise(self, A, B):
        """Add elements between matrices A and B

        Params:
        A -- N-dimensional tensor of any numerical type
        B -- N-dimensional tensor of any numerical type

        Returns:
        C -- Tensor with same dimensionality as inputs
        """

        # Trivial case
        if not isinstance(A, list) and not isinstance(B, list):
            return A + B

        assert(len(A) == len(B)), "Assert failed! Different sized operands in matsum"
        C = []
        for a, b in zip(A, B):
            C.append(self.matsum_element_wise(a, b))

        return C


    def matmul_element_wise(self, A, B):
        """Multiply elements between matrices A and B

        Params:
        A -- N-dimensional tensor of any numerical type
        B -- N-dimensional tensor of any numerical type

        Returns:
        C -- Tensor with same dimensionality as inputs
        """
        # Trivial case
        if not isinstance(A, list) and not isinstance(B, list):
            return A * B

        assert len(A) == len(B), "Assert failed! Different sized operands in matmul"
        C = []
        for a, b in zip(A, B):
            C.append(self.matmul_element_wise(a, b))

        return C

    def mat_sum(self, A):
        """Sum of all matrix cells

        Params:
        A -- Matrix A in form of [[Int]]

        Returns:
        mat_sum -- sum of matrix A elements
        """
        mat_sum = 0
        for x in range(len(A)):
            for y in range(len(A[0])):
                mat_sum = mat_sum + A[x][y]
        return mat_sum

    def matmul_native(self, A, B):
        """Perform matrix multiplication AxB=C

        Params:
        A -- Matrix A in form of [[Int]]
        B -- Matrix B in form of [[Int]]

        Returns:
        C -- Matrix C in form of [[Int]]
        """
        C = []
        for i in range(0,len(A)):
            temp=[]
            for j in range(0,len(B[0])):
                elem = 0
                for k in range(0,len(A[0])):
                    elem += A[i][k]*B[k][j]
                temp.append(elem)
            C.append(temp)
        return C


#    Logic    #
# ----------- #
#     API     #

def write(request, dla):
    print("Absolute: 0x%x  Writing request offset: %s at 0x%x, value 0x%x" % (request.absolute, str(request.type), request.offset, request.value))
    request.absolute = request.absolute & 0xFFFFFFFF # Normalize address to global address space by removing possible HPC external bit
    if int(request.absolute) >= DLA_ADDR:
        dla.set_register(request.offset, 0, 32, request.value, preserve_register=False)
    else:
        dla.handle_bank_write(request)
    dla.process()

def read(request, dla):
    original_absolute = request.absolute # Handle non-global address space addressing
    request.absolute = request.absolute & 0xFFFFFFFF # Normalize address to global address space by removing possible HPC external bit
    if int(request.absolute) >= DLA_ADDR:
        request.value = dla.get_register(request.offset, 0, 32)
    else:
        tmp = dla.handle_bank_read(request)
        request.value = tmp

    request.absolute = original_absolute # Answer to original address
    print("Absolute: 0x%x  Reading request offset: %s at 0x%x, value 0x%x" % (request.absolute, str(request.type), request.offset, request.value))

if __name__ == "__main__":
    print("Running as independent module")

    a = [[1,2],[3,4],[5,6]]
    print_matrix(a)
    print_matrix(flatten(a, 'C'))
    print_matrix(flatten(a, 'F'))

    dla = Dla()

    A_ch, A_h, A_w = 3, 5, 5
    B_ch, B_h, B_w = 2, 3, 3

    # Set input size
    dla.set_register(BUF_INPUT, BUF_CHANNELS_OFFSET, 12, A_ch - 1)
    dla.set_register(BUF_INPUT, BUF_WIDTH_OFFSET, 9, A_h - 1)
    dla.set_register(BUF_INPUT, BUF_HEIGHT_OFFSET, 9, A_w - 1)

    # Set weight size
    dla.set_register(BUF_KERNEL_0, BUF_KERNEL_0_S_CHANNELS_OFFSET, 12, B_ch - 1)
    dla.set_register(BUF_KERNEL_0, BUF_KERNEL_0_WIDTH_OFFSET, 4, B_h - 1)
    dla.set_register(BUF_KERNEL_0, BUF_KERNEL_0_HEIGHT_OFFSET, 4, B_w - 1)

    # Set banks for input and weight data
    dla.set_register(BUF_DATA_BANK, BUF_DATA_BANK_A_OFFSET, 4, 0)
    dla.set_register(BUF_DATA_BANK, BUF_DATA_BANK_B_OFFSET, 4, 8)

    # Generate 256 x 256 image
    A = [[[0,0,0,2,0], [0,1,2,1,2], [0,0,1,2,0], [1,0,0,0,2], [0,0,1,0,1]],
                 [[2,0,1,0,1], [0,0,2,2,1], [1,0,2,1,1], [2,1,2,2,1], [0,0,1,1,2]],
                 [[0,1,1,1,0], [0,2,0,1,2], [1,0,0,1,2], [1,1,1,0,0], [1,1,2,0,2]]]
    kernel_1 = [[[-1,-1,0], [-1,0,0], [-1,-1,1]],
                [[0,0,1], [1,-1,-1], [1,-1,0]],
                [[1,0,-1], [-1, 1, -1], [-1,0,-1]]]
    kernel_2 = [[[1,0,0], [-1,0,1], [0,-1,1]],
                [[0,1,-1], [-1,0,0], [0,-1,-1]],
                [[0,-1,1], [-1, -1, -1], [0,1,0]]]
    B = [kernel_1, kernel_2]

    print_matrix(A[0], "A0:")

    # A = separate_channels(A) # CHW to 2D
    # B = separate_channels(B)
    C = dla.mac.conv2d(A, B)
    for (i,c) in enumerate(C):
        print_matrix(c, "C{}".format(i))

    # Write input data to data bank
    A = flatten(A)
    dla.set_input_data(A)

    # Write weight data to data bank
    B = flatten(B)
    dla.set_weight_data(B)

    # Initialization ready
    dla.set_register(HANDSHAKE, HANDSHAKE_BUFFER_ENABLE_OFFSET, 1, 0x1) # Data buffer
    dla.set_register(HANDSHAKE, HANDSHAKE_MAC_ENABLE_OFFSET, 1, 0x1) # DLA array
    dla.set_register(HANDSHAKE, HANDSHAKE_BYPASS_ENABLE_OFFSET, 1, 0x1) # Post processor

    # Enable PP
    dla.set_register(PP_CTRL, PP_SELECT_OFFSET, 1, 1)
    dla.set_register(PP_CTRL, ACTIVE_MODE_OFFSET, 2, 0)
    dla.set_register(HANDSHAKE, HANDSHAKE_BIAS_ENABLE_OFFSET, 1, 1)
    dla.set_register(HANDSHAKE, HANDSHAKE_ACTIVE_ENABLE_OFFSET, 1, 1)

    ## Mac clip
    dla.set_register(MAC_CTRL, MAC_CLIP_OFFSET, 5, 8)
    # PP clip
    dla.set_register(PP_CTRL, PP_CLIP_OFFSET, 5, 8)

    # Input data is ready
    dla.set_register(BUF_CTRL, READ_A_VALID_OFFSET, 1, 0x1) # All weight data ready
    dla.set_register(BUF_CTRL, READ_B_VALID_OFFSET, 1, 0x1) # All input data ready

    dla.process()

else:

    if request.isInit:
        # Supress all python print in CI
        is_ci = os.environ.get("CI")
        if not is_ci == None:
            sys.stdout = open(os.devnull, 'w')

        # Initialized DLA
        dla = Dla()
        print("%s initialized" % NAME)
        self.NoisyLog("%s initialized" % NAME)
    elif request.isRead:
        read(request, dla)
    elif request.isWrite:
        write(request, dla)
    else:
        self.NoisyLog("Bad request: %s at 0x%x, value 0x%x" % (str(request.type), request.offset, request.value))
        print("Bad request: %s at 0x%x, value 0x%x" % (str(request.type), request.offset, request.value))
