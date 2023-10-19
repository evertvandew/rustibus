
#![no_std]


pub mod RustIBus {
    #[warn(non_snake_case)]
    use core::ops::Index;
    use core::iter::ExactSizeIterator;


    const SET: u8 = 0x40;
    const DISCOVER: u8 = 0x80;
    const TYPE: u8 = 0x90;
    const VALUE: u8 = 0xa0;

    #[derive(PartialEq, Debug, Clone, Copy)]
    #[repr(u8)]
    pub enum IBusSensor {
        INTV = 0x00,
        TEMP = 0x01,
        RPM = 0x02,
        EXTV = 0x03,
        PRESS = 0x41,
        SERVO = 0xfd
    }

    #[derive(PartialEq, Debug, Clone, Copy)]
    #[repr(u8)]
    pub enum IBusSensorLength {
        Short = 0x02,
        Long = 0x04,
    }


    #[derive(PartialEq, Debug)]
    pub enum IBusMsg {
        DiscoveryRequest(u8),
        DiscoveryResponse(u8),
        SetMsg([u16; 14]),
        TypeRequest(u8),
        TypeResponse(u8, IBusSensor, IBusSensorLength),
        ValueRequest(u8),
        ValueResponseShort(u8, u16),
        ValueResponseLong(u8, u32)
    }


    const MAX_LENGTH: u8 = 0x20;
    const MIN_LENGTH: u8 = 0x04;


    fn checkForResync<T: Index<usize, Output=u8> + ExactSizeIterator>(buffer: &T) -> bool {
        /// Check if the buffer contains a valid IBus message.
        /// Checks for buffer length, valid command code and the CRC.
        /// Returns true if a single byte can be consumed, to try and resync.
        // Check for a correct length character
        if (buffer.len() > 0) && ((buffer[0] < MIN_LENGTH) || (buffer[0] > MAX_LENGTH)) {
            return true;
        }

        // If enough bytes have been received, check the message contents
        if (buffer.len() as u8) < buffer[0] {
            // We can't check the CRC yet
            return false;
        }

        // The second byte should be a known command code
        // The high nibble is the command, the low nibble the address.
        match buffer[1] & 0xf0 {
            SET => (),
            TYPE | VALUE | DISCOVER => { if buffer[0] != 0x04 { return true; } },
            _ => {return true;}
        };

        // Check the CRC
        let mut crc = 0xffffu16;
        for i in 0..buffer[0] - 2 {
            crc -= buffer[i as usize] as u16
        }
        if (crc >> 8) as u8 != buffer[(buffer[0] - 1) as usize] || (crc & 0xff) as u8 != buffer[(buffer[0] - 2) as usize] {
            return true;
        }
        return false;
    }


    fn popSetMsg<T: Index<usize, Output=u8> + ExactSizeIterator>(length: u8, buffer: &T) -> IBusMsg {
        // Determine how many sensors are being set
        // For now, assume there are 20H bytes / 10 actuator values
        let mut data = [0u16; 14];
        for i in 0..((length / 2 - 2) as usize) {
            data[i] = buffer[2 + 2 * i] as u16;
            data[i] += ((buffer[3 + 2 * i]) as u16) << 8;
        }
        IBusMsg::SetMsg(data)
    }

    pub fn popIBusMsg<T: Index<usize, Output=u8> + ExactSizeIterator>(buffer: &T) -> (Option<IBusMsg>, u8) {
        // Find a correct message
        while checkForResync(buffer) {
            // Remove the first character in an attempt to re-synchronize.
            return (None, 1);
        }

        // Ensure a message has been received. If not, ask for more.
        if buffer.len() < 2 || (buffer.len() as u8) < buffer[0usize] {
            return (None, 0);
        }

        // A message with a correct length, CRC and command code has been detected. Handle it.
        let length = buffer[0];
        let cmnd = buffer[1] & 0xf0;
        let addr = buffer[1] & 0x0f;
        let msg = match cmnd {
            DISCOVER => Some(IBusMsg::DiscoveryRequest(addr)),
            SET => Some(popSetMsg(length, buffer)),
            TYPE => Some(IBusMsg::TypeRequest(addr)),
            VALUE => Some(IBusMsg::ValueRequest(addr)),
            _ => None
        };
        return (msg, length);
    }


// fn pushMsg(msg: &[u8], buffer: &mut Buffer) {
//     let length = (msg.len() & 0xff) as u8 + 3;
//     if length < MIN_LENGTH || length > MAX_LENGTH { return; }
//     let mut crc: u16 = 0xffff;
//     buffer.push(length);
//     crc -= length as u16;
//     for b in msg {
//         buffer.push(*b);
//         crc -= *b as u16;
//     }
//     buffer.push((crc & 0xff) as u8);
//     buffer.push((crc >> 8) as u8);
// }
//
//
// pub fn pushIBusMsg(msg: &IBusMsg, buffer: &mut Buffer) {
//     match msg {
//         IBusMsg::DiscoveryResponse(addr) =>
//             pushMsg(&[DISCOVER + addr], buffer),
//         IBusMsg::TypeResponse(addr, sensortype, length) =>
//             pushMsg(&[TYPE+addr, *sensortype as u8, *length as u8], buffer),
//         IBusMsg::ValueResponseShort(addr, value) =>
//             pushMsg(&[VALUE+addr, (value&0xff) as u8, (value>>8) as u8], buffer),
//         IBusMsg::ValueResponseLong(addr, value) =>
//             pushMsg(&[VALUE+addr, (value&0xff) as u8, ((value>>8) & 0xff) as u8, ((value>>16) & 0xff) as u8, ((value>>24) & 0xff) as u8], buffer),
//         _ => panic!()
//     }
// }
}


mod deque;

#[cfg(test)]
mod tests {
    //mod super::deque;

    use super::RustIBus::*;
    use super::deque::Deque;

    type Buffer = Deque::<64>;


    #[test]
    fn test_resync() {
        let mut buffer = Buffer::new();
        // Test a wrong command code followed by a correct message
        buffer.load(&[0x04, 0x04, 0x81, 0x7a, 0xff]);
        assert_eq!(popIBusMsg(&mut buffer), (None, 1));
        // Test a message that is too short
        buffer.clear();
        buffer.load(&[0x00, 0x04, 0x81, 0x7a, 0xff]);
        assert_eq!(popIBusMsg(&mut buffer), (None, 1));
        // Test a message that is too long
        buffer.clear();
        buffer.load(&[0x21, 0x04, 0x81, 0x7a, 0xff]);
        assert_eq!(popIBusMsg(&mut buffer), (None, 1));
        // Test a message that is not complete
        buffer.clear();
        buffer.load(&[0x04, 0x81, 0x7a]);
        assert_eq!(popIBusMsg(&mut buffer), (None, 0));
        // Test a message with a wrong CRC check
        buffer.clear();
        buffer.load(&[0x04, 0x81, 0x7b, 0xff]);
        assert_eq!(popIBusMsg(&mut buffer), (None, 1));
        // Test a message with a wrong command code
        buffer.clear();
        buffer.load(&[0x04, 0x11, 0xea, 0xff]);
        assert_eq!(popIBusMsg(&mut buffer), (None, 1));
        // Test a good message can still be read
        buffer.clear();
        buffer.load(&[0x04, 0x81, 0x7a, 0xff]);
        assert_eq!(popIBusMsg(&mut buffer), (Some(IBusMsg::DiscoveryRequest(0x1)), 4));
    }

    #[test]
    fn test_setmsg() {
        let mut buffer = Buffer::new();
        buffer.load(&[0x20, 0x40, 0xDB, 0x05, 0xDC, 0x05, 0x54, 0x05,
                          0xDC, 0x05, 0xE8, 0x03, 0xD0, 0x07, 0xD2, 0x05,
                          0xE8, 0x03, 0xDC, 0x05, 0xDC, 0x05, 0xDC, 0x05,
                          0xDC, 0x05, 0xDC, 0x05, 0xDC, 0x05, 0xDA, 0xF3]);
        assert_eq!(popIBusMsg(&mut buffer), (Some(IBusMsg::SetMsg([
            0x5DB, 0x5Dc, 0x554, 0x5DC, 0x3E8, 0x7D0, 0x5D2,
            0x3E8, 0x5DC, 0x5DC, 0x5DC, 0x5DC, 0x5DC, 0x5DC])), 0x20));
    }

    #[test]
    fn test_parseshortmsgs() {
        let mut buffer = Buffer::new();
        buffer.load(&[0x04, 0x81, 0x7a, 0xff]);
        assert_eq!(popIBusMsg(&mut buffer), (Some(IBusMsg::DiscoveryRequest(0x01)), 4));
        buffer.clear();
        buffer.load(&[0x04, 0x92, 0x69, 0xff]);
        assert_eq!(popIBusMsg(&mut buffer), (Some(IBusMsg::TypeRequest(0x02)), 4));
        buffer.clear();
        buffer.load(&[0x04, 0xa3, 0x58, 0xff]);
        assert_eq!(popIBusMsg(&mut buffer), (Some(IBusMsg::ValueRequest(0x03)), 4));
        buffer.clear();
    }

    // #[test]
    // fn test_pushshortmsgs() {
    //     let mut buffer = Buffer::new();
    //     pushIBusMsg(&IBusMsg::DiscoveryResponse(0x01), &mut buffer);
    //     assert_eq!(buffer.iter().collect::<Vec<u8>>(), [0x04, 0x81, 0x7a, 0xff]);
    //     buffer.clear();
    //
    //     pushIBusMsg(&IBusMsg::TypeResponse(0x02, IBusSensor::PRESS, IBusSensorLength::Long), &mut buffer);
    //     assert_eq!(buffer.iter().collect::<Vec<u8>>(), [0x06, 0x92, 0x41, 0x04, 0x22, 0xff]);
    //     buffer.clear();
    //
    //     pushIBusMsg(&IBusMsg::ValueResponseLong(0x03, 0x12345678), &mut buffer);
    //     assert_eq!(buffer.iter().collect::<Vec<u8>>(), [0x08, 0xa3, 0x78, 0x56, 0x34, 0x12, 0x40, 0xfe]);
    // }
}
