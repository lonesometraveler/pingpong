# pingpong

A ping-pong buffer implementation for embedded Rust applications

## description

Ping-pong or double buffering is a mechanism for allowing a data stream to be written to one buffer whilst data is read from the other in a fashion that the read and write operations do not collide.
This library is a simple implementation of this idea and is a **work in progress**. I am currently using it for buffering data streams originating from SDMMC controllers on embedded platforms, but it could be used for other things. 
