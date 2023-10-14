

# Functionality

* Receiving setpoints for servos etc
* Sending sensor values back
* Going to "safe" values when connection is lost
* Checking integrity of communication (CRC)




# Code Dependencies
We use the [stm32f4xx](https://docs.rs/stm32f4xx-hal/latest/stm32f4xx_hal) hal. 
This implements the excellent and well-documented Rust [embedded_hal](https://docs.rs/embedded-hal/latest/embedded_hal/).
This depends on the [stm32f4](https://docs.rs/stm32f4/latest/stm32f4) crate, 
which is generated from SVD device descriptions of the STM32F4 family.
This generation is done with the `svd2rust` tool.

In case you use other families of STM32 devices, they all have similarly named HAL packages.

To see the complete dependency tree of the software, execute `cargo tree` in the root directory.