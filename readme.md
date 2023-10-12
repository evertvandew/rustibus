

# Functionality

* Receiving setpoints for servos etc
* Sending sensor values back
* Going to "safe" values when connection is lost
* Checking integrity of communication (CRC)




# Code Dependencies
We use the stm32f4xx hal. 
This implements the excellent and well-documented Rust embedded_hal,
and is generated from SVD device descriptions of the STM32F4 family.

https://docs.rs/stm32f4xx-hal/latest/stm32f4xx_hal/
