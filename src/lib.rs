#![doc = include_str!("../README.md")]
#![no_std]

/// Pingpong or double buffering is useful for buffering tasks requiring simultaneous reading and writing.
/// While one buffer is being written to, the other can be read from and vice versa.
/// This implementation uses two buffers, "active" and "reserve."
/// We write to the active buffer and read from the reserve buffer.
pub struct PingpongBuffer<const N: usize, T> {
    /// The first internal buffer
    buffer_a: [T; N],
    /// The second internal buffer
    buffer_b: [T; N],
    /// The active buffer index
    active_index: usize,
    /// Active buffer
    active_buffer: Buffer,
    /// An enum to indicate that the reserve buffer is full or not
    reserve_capacity: BufferCapacity,
}

/// Errors used in this crate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingpongBufferError {
    /// The data is larger than the internal double buffers can process
    /// The PingpongBuffer cannot datasets larger than 2N to its internal buffers
    Overflow,
    /// The reserve data buffer is full and cannot be written to. Occurs when appending data
    /// to the active buffer and the reserve buffer is toggled to become the active buffer
    ReserveFull,
}

/// Flag to indicate active buffer
#[derive(Debug, PartialEq, Eq)]
pub enum Buffer {
    A,
    B,
}

impl Buffer {
    /// Switch active buffer
    fn toggle(&self) -> Self {
        match self {
            Buffer::A => Buffer::B,
            Buffer::B => Buffer::A,
        }
    }
}

/// Enum to express if buffer is full or not
#[derive(Debug, PartialEq, Eq)]
pub enum BufferCapacity {
    Full,
    NotFull,
}

impl<const N: usize, T> Default for PingpongBuffer<N, T>
where
    T: Default + Copy,
{
    fn default() -> Self {
        Self {
            buffer_a: [T::default(); N],
            buffer_b: [T::default(); N],
            active_index: 0,
            active_buffer: Buffer::A,
            reserve_capacity: BufferCapacity::NotFull,
        }
    }
}

impl<const N: usize, T> PingpongBuffer<N, T>
where
    T: Default + Copy,
{
    /// Initialize an instance of the PingpongBuffer. This is the same as calling `PingpongBuffer::<N, T>::DEFAULT`
    pub fn new() -> Self {
        PingpongBuffer::<N, T>::default()
    }

    /// Is the actively written buffer empty
    pub fn is_empty(&self) -> bool {
        self.active_index == 0
    }

    /// Is the actively written buffer more than half full
    pub fn is_half_full(&self) -> bool {
        self.active_index >= (N / 2)
    }

    /// Is the reserve buffer full and ready for reading
    pub fn is_reserve_full(&self) -> bool {
        self.reserve_capacity == BufferCapacity::Full
    }

    /// Clears the Pingpong buffer to return it back into its default state
    pub fn clear(&mut self) {
        self.buffer_a = [T::default(); N];
        self.buffer_b = [T::default(); N];
        self.active_index = 0;
        self.active_buffer = Buffer::A;
        self.reserve_capacity = BufferCapacity::NotFull;
    }

    /// Read out the remainding data from the active buffer
    /// Useful in circumstances in which the buffering process needs to end, and there
    /// isn't enough data to toggle between the active and reserve buffers
    pub fn flush(&mut self) -> ([T; N], usize) {
        let buff = match self.active_buffer {
            Buffer::A => &self.buffer_a,
            Buffer::B => &self.buffer_b,
        };

        let mut data: [T; N] = [T::default(); N];
        let remainder = self.active_index;
        data[0..remainder].copy_from_slice(&buff[0..remainder]);
        self.active_index = 0;
        (data, remainder)
    }

    /// Read the data from the reserve buffer.
    /// If the reserve buffer is not yet full, this function will return Option::None
    /// Once the bytes are read from, this will allow the reserve buffer to be toggled into
    /// the active buffer
    pub fn read(&mut self) -> Option<[T; N]> {
        if let BufferCapacity::Full = self.reserve_capacity {
            self.reserve_capacity = BufferCapacity::NotFull;
            match self.active_buffer {
                Buffer::A => Some(self.buffer_b),
                Buffer::B => Some(self.buffer_a),
            }
        } else {
            None
        }
    }

    /// The position within the active buffer
    pub fn position(&self) -> usize {
        self.active_index
    }

    /// Push an element to the active buffer
    /// If the active buffer fills to maximum capacity, then the active and reserve buffers
    /// are switched, allowing the remaining data to be written to the reserve (now active) buffer
    /// This switch can only happen if the data in the reserve buffer has been successfully read
    pub fn push(&mut self, element: T) -> Result<BufferCapacity, PingpongBufferError> {
        self.append(&[element])
    }

    /// Append data to the active buffer
    /// If the active buffer fills to maximum capacity, then the active and reserve buffers
    /// are switched, allowing the remaining data to be written to the reserve (now active) buffer
    /// This switch can only happen if the data in the reserve buffer has been successfully read
    pub fn append(&mut self, data: &[T]) -> Result<BufferCapacity, PingpongBufferError> {
        // get the active buffer
        let buff = match self.active_buffer {
            Buffer::A => &mut self.buffer_a,
            Buffer::B => &mut self.buffer_b,
        };

        // determine how many bytes we can append to the buffer
        let capacity = buff.len() - self.active_index;

        // number of bytes appended to the buffer
        let transferred = core::cmp::min(data.len(), capacity);

        // copy the data to the buffer
        buff[self.active_index..(self.active_index + transferred)]
            .copy_from_slice(&data[..transferred]);

        // increment the index
        self.active_index += transferred;

        // We are at the end of the buffer
        if self.active_index == buff.len() {
            if let BufferCapacity::Full = self.reserve_capacity {
                // We are attempting to switch the reserve->active buffer,
                // but the reserve buffer is still full of data that has not been read
                return Err(PingpongBufferError::ReserveFull);
            }
            // Toggle that the reserve buffer is full, and can be read
            self.reserve_capacity = BufferCapacity::Full;
            // Reset the active index for writing
            self.active_index = 0;
            // Toggle the buffer
            self.active_buffer = self.active_buffer.toggle();
            // There is left over data that needs to be transferred
            if transferred != data.len() {
                let buff = match self.active_buffer {
                    Buffer::A => &mut self.buffer_a,
                    Buffer::B => &mut self.buffer_b,
                };

                // copy the remainder into the other buffer
                let remainder = data.len() - transferred;
                if remainder > N {
                    return Result::Err(PingpongBufferError::Overflow);
                }
                buff[..remainder].copy_from_slice(&data[transferred..]);
                self.active_index += remainder;
            }
            // The buffers have been toggled
            return Ok(BufferCapacity::Full);
        }
        // The buffers have not been toggled
        Ok(BufferCapacity::NotFull)
    }
}

#[cfg(test)]
mod tests {
    /// The pingpong buffer size used for testing purposes
    const BUFFER_SIZE: usize = 1024;

    use crate::{BufferCapacity, PingpongBuffer, PingpongBufferError};

    #[test]
    fn is_empty() {
        let mut buff = PingpongBuffer::<BUFFER_SIZE, u8>::default();
        assert!(buff.is_empty());
        buff.append(&[0x01; 32]).unwrap();
        assert!(!buff.is_empty());
    }

    #[test]
    fn is_half_full() {
        let mut buff = PingpongBuffer::<BUFFER_SIZE, u8>::default();
        assert!(!buff.is_half_full());
        buff.append(&[0x01; BUFFER_SIZE / 2]).unwrap();
        assert!(buff.is_half_full());
        buff.append(&[0x01; 5]).unwrap();
        assert!(buff.is_half_full());
    }

    #[test]
    fn append_with_toggle() {
        let mut buff = PingpongBuffer::<BUFFER_SIZE, u8>::default();
        let capacity = buff.append(&[0x01; BUFFER_SIZE]).unwrap();
        assert_eq!(capacity, BufferCapacity::Full);
        assert_eq!(buff.active_index, 0);
        assert_eq!(buff.reserve_capacity, BufferCapacity::Full);

        let mut buff = PingpongBuffer::<BUFFER_SIZE, u8>::default();
        let capacity = buff.append(&[0x01; BUFFER_SIZE + 6]).unwrap();
        assert_eq!(capacity, BufferCapacity::Full);
        assert_eq!(buff.active_index, 6);
        assert_eq!(buff.reserve_capacity, BufferCapacity::Full);
    }

    #[test]
    fn append_without_toggle() {
        let mut buff = PingpongBuffer::<BUFFER_SIZE, u32>::default();
        let capacity = buff.append(&[0x01122311; BUFFER_SIZE / 2]).unwrap();
        assert_eq!(capacity, BufferCapacity::NotFull);
        assert_eq!(buff.active_index, BUFFER_SIZE / 2);
        assert_eq!(buff.reserve_capacity, BufferCapacity::NotFull);
    }

    #[test]
    fn append_overflow() {
        let mut buff = PingpongBuffer::<BUFFER_SIZE, u8>::default();
        let result = buff.append(&[0x01; (BUFFER_SIZE * 2)]);
        assert!(result.is_ok());

        let mut buff = PingpongBuffer::<BUFFER_SIZE, u8>::default();
        let result = buff.append(&[0x01; (BUFFER_SIZE * 2) + 1]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PingpongBufferError::Overflow);
    }

    #[test]
    fn append_reserve_full() {
        let mut buff = PingpongBuffer::<BUFFER_SIZE, u8>::default();
        // Fill up the reserve buffer
        let result = buff.append(&[0x01; BUFFER_SIZE]);
        assert!(result.is_ok());
        // Fill up the active buffer up until the point of toggling
        let result = buff.append(&[0x01; BUFFER_SIZE - 1]);
        assert!(result.is_ok());
        // Append more data, triggering the toggle event, but the reserve buffer wasnt read
        let result = buff.append(&[0x01; 1]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PingpongBufferError::ReserveFull);
    }

    #[test]
    fn read_reserve_not_full() {
        let mut buff = PingpongBuffer::<BUFFER_SIZE, u8>::default();
        assert_eq!(buff.read(), None);
        buff.append(&[0x01; BUFFER_SIZE - 1]).unwrap();
        assert_eq!(buff.read(), None);
    }

    #[test]
    fn read_reserve_full() {
        let mut buff = PingpongBuffer::<BUFFER_SIZE, u16>::default();
        buff.append(&[0x0100; BUFFER_SIZE]).unwrap();
        let result = buff.read().unwrap();
        assert_eq!(result.len(), BUFFER_SIZE);
        assert!(result.iter().all(|v| *v == 0x0100));
    }

    #[test]
    fn reserve_is_not_full() {
        let mut buff = PingpongBuffer::<BUFFER_SIZE, u16>::default();
        buff.append(&[0x0100; BUFFER_SIZE - 1]).unwrap();
        assert!(!buff.is_reserve_full());
    }

    #[test]
    fn reserve_is_full() {
        let mut buff = PingpongBuffer::<BUFFER_SIZE, u16>::default();
        buff.append(&[0x0100; BUFFER_SIZE]).unwrap();
        assert!(buff.is_reserve_full());
    }
}
