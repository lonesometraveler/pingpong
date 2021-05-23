//! # pingpong
//! A pingpong or double buffer for embedded and no_std applications

#![deny(warnings)]
#![no_std]
#![allow(dead_code)]

/// Pingpong or double buffering is useful for performing buffering tasks that require similtaneous
/// reading and writing. While one buffer is being written to, the other can be read from and visa versa.
/// In this implementation, the buffer that is being written to is known as the "active" buffer
/// and the buffer being read from is the "reserve" buffer
pub struct PingpongBuffer<const N: usize> {
    /// The first internal buffer
    buffer_a: [u8; N],
    /// The second internal buffer
    buffer_b: [u8; N],
    /// The active buffer index
    active_index: usize,
    /// A toggle to determine which of the internal buffers is active and which is reserve
    active_toggle: bool,
    /// A flag to indicate that the reserve buffer is full
    is_reserve_full: bool,
}

impl<const N: usize> PingpongBuffer<N> {
    /// The default PingpongBuffer
    pub const DEFAULT: PingpongBuffer<N> = PingpongBuffer::<N> {
        buffer_a: [0; N],
        buffer_b: [0; N],
        active_index: 0,
        active_toggle: true,
        is_reserve_full: false,
    };

    /// Is the actively written buffer empty
    pub fn is_empty(&self) -> bool {
        self.active_index == 0
    }

    /// Is the actively written buffer more than half full
    pub fn is_half_full(&self) -> bool {
        self.active_index >= (N / 2)
    }

    /// Clears the Pingpong buffer to return it back into its default state
    pub fn clear(&mut self) {
        self.buffer_a = [0; N];
        self.buffer_b = [0; N];
        self.active_index = 0;
        self.active_toggle = true;
        self.is_reserve_full = false;
    }

    /// Read out the remainding data from the active buffer
    /// Useful in circumstances in which the buffering process needs to end, and there
    /// isnt enough data to toggle between the active and reserve buffers
    pub fn flush(&mut self) -> ([u8; N], usize) {
        // Get the active buffer
        let buff = if self.active_toggle {
            &self.buffer_a
        } else {
            &self.buffer_b
        };

        let mut data: [u8; N] = [0; N];
        let remainder = self.active_index;
        data[0..remainder].copy_from_slice(&buff[0..remainder]);
        self.active_index = 0;
        (data, remainder)
    }

    /// Read the data from the reserve buffer.
    /// If the reserve buffer is not yet full, this function will return Option::None
    /// Once the bytes are read from, this will allow the reserve buffer to be toggled into
    /// the active buffer
    pub fn read(&mut self) -> Option<&[u8; N]> {
        if !self.is_reserve_full {
            // For the sake of simplicity to begin with, return None
            // if the reserve buffer is not ready
            return Option::None;
        }
        let buff = if self.active_toggle {
            Option::Some(&self.buffer_b)
        } else {
            Option::Some(&self.buffer_a)
        };

        // After we have read the bytes, then they can be cleared out allowing
        // The reserve buffer to become to active buffer
        self.is_reserve_full = false;

        return buff;
    }

    /// Append data to the active buffer
    /// If the active buffer fills to maximum capacity, then the active and reserve buffers
    /// are switched, allowing the remainding data to be written to the reserve (now active) buffer
    /// This switch can only happen if the data in the reserve buffer has been successfully read
    pub fn append(&mut self, data: &[u8]) -> bool {
        // get the active buffer
        let buff = if self.active_toggle {
            &mut self.buffer_a
        } else {
            &mut self.buffer_b
        };

        // determine how many bytes we can append to the buffer
        let capacity = buff.len() - self.active_index;

        // iterate through the data, to add it to our buffer at the index
        let start = self.active_index;
        for i in 0..data.len() {
            if i >= capacity {
                break; // cannot fill anymore into this buffer
            }
            buff[start + i] = data[i];
            self.active_index += 1;
        }

        // number of bytes appended to the buffer
        let transferred = if capacity > data.len() {
            data.len()
        } else {
            capacity
        };

        // We are at the end of the buffer
        if self.active_index == buff.len() {
            if self.is_reserve_full {
                // We are attempting to switch the reserve->active buffer,
                // but the reserve buffer is still full of data that has not be read
                panic!("The reserve buffer cannot be written to as it is still full. Clear it by reading the data");
            }
            // Toggle that the reserve buffer is full, and can be read
            self.is_reserve_full = true;
            // Reset the active index for writing
            self.active_index = 0;
            // Toggle the buffer
            self.active_toggle = !self.active_toggle;
            // There is left over data that needs to be transferred
            if transferred != data.len() {
                let buff = if self.active_toggle {
                    &mut self.buffer_a
                } else {
                    &mut self.buffer_b
                };

                // copy the remainder into the other buffer
                let remainder = data.len() - transferred;
                if remainder > N {
                    panic!("The data is larger than the double buffer can process");
                }
                for i in 0..remainder {
                    buff[i] = data[transferred + i];
                    self.active_index += 1;
                }
            }
            // The buffers have been toggled
            return true;
        }
        // The buffers have not been toggled
        return false;
    }
}
