

# RustIBus

RustIBus is a simple parser for messages used in Radio Control.

Some RC receivers provide an IBus output, though which digital values for the servo channels
sent by the RC controller are given. Advanced controller of remote devices can use these digital
signals to implement specific control functions. Also, some RC receivers support the sending
of telemetery data, using the same protocol.

This library provides the module RustIBus, which contains the function `popIBusMsg`. 
This function receives a slice of bytes `u8`, and checks if this set contains an IBus Message.
It returns a tuple `(Option<IBusMsg>, u8)`, with the second byte indicating how many bytes the
slice can be advanced, if at all.

The module depends on nothing except the Rust `core`. It is `no_std`.

The parser does not assume the protocol is synchronized. It does a number of checks before reading a message:

* Is the message size (the first byte in the message) valid? Messages must be between 4 and 32 bytes long. If not, it assumes it is not synchronized.
* Does the slice contain the whole message? If not, it returns `(None, 0)`.
* Is the message type valid (the second byte). If not, it assumes it is not synchronized.
* Is the checksum correct? If not, it assumes it is not synchronized.

When the parser concludes it is not synchronized, it returns `(None, 1)`, 
indicating that the first byte in the buffer can be skipped.
The parse will continue skipping single bytes until it finds a message that matches all criteria.

If a message is successfully parsed, it returns `(Some<msg>, <len>)`.
`<len>` is the number of bytes that can be consumed from the buffer.
Message data is copied into the message, so the buffer can be released immediately.

## Setpoint Message
The most important message is `IBusMsg::SetMsg`. It contains 14 `u16` words, representing 14 servo channels.
Depending on the RC controller and its configuration, 4 or more channels are actually used starting at the first.

Setpoint values vary between 1000 and 2000, with 1500 being the center point.

My controller transmits more than 100 Setpoint messages per second. The IBus transmits at 115200 Baud.

## Deque buffer
One problem with the IBus protocol is that it has no specific `SOM` or `EOM`
character, so it is hard to determine when a message is supposed to start. Given the length of the Set message
and the high repeat frequency, the protocol is sending bytes more than 50% of the time.
So chances are high that the first byte that is read, is not actually the start of a message.

The whole message must be read to check the checksum, and that is the only way to determine if a correct message has been received.
If the message is incorrect, the buffer needs to advance by only a single byte, as it needs to look for the
correct start of a message. So, the parser must use a buffer where all bytes can be accessed randomly without consuming them.
For this, the protocol assumes the buffer implements the [`Index`](https://doc.rust-lang.org/core/ops/trait.Index.html) and [`ExactSizeIterator`](https://doc.rust-lang.org/core/iter/trait.ExactSizeIterator.html) traits.

Probably, I could have tweaked e.g. the [fring](https://docs.rs/fring/latest/fring/) buffer to this end. However, I want to learn coding in Rust and
implemented a circular buffer myself with the desired traits. It is in the example, as `deque.rs`.

## Example
The `examples` directory contains a single example, written for a STM32F446 MCU.
It reads the third servo channel and drives a PWM port according to its value.
It makes use of the wonderful [stm32f4xx-hal](https://docs.rs/stm32f4xx-hal/latest/stm32f4xx_hal/) with the [rtic](https://docs.rs/cortex-m-rtic/latest/rtic/) framework.

The example uses two buffers: a standard [heapless::spsc](https://docs.rs/heapless/latest/heapless/spsc/index.html) channel to quickly read the serial data in its interrupt,
and a second buffer to parse the messages. 
