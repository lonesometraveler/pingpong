# pingpong

A ping-pong buffer implementation for embedded Rust applications

## description

Ping-pong or double buffering is a mechanism for allowing a data stream to be written to one buffer whilst data is read from the other in a fashion that the read and write operations do not collide.
In this implementation, the buffer that is being written to is known as the `active` buffer and the buffer being read from is the `reserve` buffer.

## note

This library is a simple implementation of the double buffering concept and is a **work in progress**. I am currently using it for buffering data streams to and from SDMMC controllers on embedded platforms, but it could be used for other things. Thread safety have not yet been implemented. So please take care using this library.

## example

```rust
// Create the pingpong buffer with both active and reserve buffers being 1024 elements long
let mut buff = PingpongBuffer::<1024>::new();

// In one context get a stream of data and append it to our buffer
let data: [u8; 128] = [0x01; 128];
let mut is_reserve_full = false;

while !is_reserve_full {
    // Once the active buffer (1024 bytes) is full, it is converted into a reserve buffer
    // which allows for safe reading
    is_reserve_full = buff.append(&data).unwrap();
}

// Similtaneously in another context read data from the reserve buffer. 
// This will not affect / collide the data being written to in the active buffer.
if buff.is_reserve_full() {
    let data: [u8; 1024] = buff.read().unwrap();
}
```