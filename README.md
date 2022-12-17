# pingpong

A ping-pong buffer implementation for embedded Rust applications

## Description

Ping-pong or double buffering allows a data stream to be written to one buffer while data is read from the other so that the read and write operations do not collide. In this implementation, the buffer being written to is called "active" and the buffer being read from is called "reserve".


## Example

```rust
// Create the pingpong buffer with both active and reserve buffers being 1024 elements long
let mut buff = PingpongBuffer::<1024, u8>::new();

// Get a stream of data and append it to the active buffer.
// Once the active buffer (1024 bytes) is full, it is converted into a reserve buffer.
let data: [u8; 128] = [0x01; 128];
while let Ok(BufferCapacity::NotFull) = buff.append(&data) {}

// Read data from the reserve buffer.
if buff.is_reserve_full() {
    let data: [u8; 1024] = buff.read().unwrap();
    println!("data: {:?}", data);
}
```
