
mod deque;
use deque::Deque;

type Buffer = Deque::<u8, 64>;

const SET:u8      = 0x40;
const DISCOVER:u8 = 0x80;
const TYPE:u8     = 0x90;
const VALUE:u8    = 0xa0;

#[repr(u8)]
enum IBusSensors {
    INTV  = 0x00,
    TEMP  = 0x01,
    RPM   = 0x02,
    EXTV  = 0x03,
    PRESS = 0x41,
    SERVO = 0xfd
}


#[derive(PartialEq, Debug)]
pub enum IBusMsg {
    DiscoveryRequest(u8),
    DiscoveryResponse(u8),
    SetMsg([u16; 14]),
    TypeRequest(u8),
    TypeResponse(u8, u8, u8),
    ValueRequest(u8),
    ValueResponseShort(u8, u16),
    ValueResponseLong(u8, u32)
}




const MAX_LENGTH: u8 = 0x20;
const MIN_LENGTH: u8 = 0x04;


fn checkForResync(buffer: &Buffer) -> bool {
    // Check for a correct length character
    if buffer.len() > 0 && buffer[0] < MIN_LENGTH && buffer[0] > MAX_LENGTH {
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
        SET | TYPE | VALUE | DISCOVER => (),
        _    => {return true;}
    };

    // Check the CRC
    let mut count = 0u16;
    for i in 0..buffer[0] {
        count += buffer[i as usize] as u16
    }
    count -= 0xffff;
    if (count >> 8) as u8 != buffer[(buffer[0]-1) as usize] || (count & 0xff) as u8 != buffer[(buffer[0]-2) as usize] {
        return true;
    }
    return false;
}


fn parseSetMsg(buffer: &Buffer) -> IBusMsg {
    // Determine how many sensors are being set
    // For now, assume there are 20H bytes / 10 actuator values
    let mut data = [0u16; 14];
    for i in 0..14 {
        data[i] = buffer[(2+2*i) as usize] as u16 + (buffer[(3+2*i) as usize] as u16) << 8;
    }
    IBusMsg::SetMsg(data)
}

pub fn parse(buffer: &mut Buffer) -> Option<IBusMsg> {
    // Find a correct message
    while checkForResync(buffer) {
        // Remove the first character in an attempt to re-synchronize.
        buffer.pop();
    }

    // Ensure a message has been received. If not, ask for more.
    if buffer.len() < 2 || (buffer.len() as u8) < buffer[0usize] {
        return None;
    }

    // A message with a correct length, CRC and command code has been detected. Handle it.
    let addr = buffer[1] & 0x0f;
    match buffer[1] & 0xf0 {
        DISCOVER => Some(IBusMsg::DiscoveryRequest(addr)),
        SET      => Some(parseSetMsg(buffer)),
        TYPE     => Some(IBusMsg::TypeRequest(addr)),
        VALUE    => Some(IBusMsg::ValueRequest(addr)),
        _        => None
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setmsg() {
        let mut buffer = Buffer::new();
        buffer.load(&[0x20, 0x40, 0xDB, 0x05, 0xDC, 0x05, 0x54, 0x05,
                          0xDC, 0x05, 0xE8, 0x03, 0xD0, 0x07, 0xD2, 0x05,
                          0xE8, 0x03, 0xDC, 0x05, 0xDC, 0x05, 0xDC, 0x05,
                          0xDC, 0x05, 0xDC, 0x05, 0xDC, 0x05, 0xDA, 0xF3]);
        assert_eq!(parse(&mut buffer), Some(IBusMsg::SetMsg([
            0x5DB, 0x5Dc, 0x554, 0x5DC, 0x3E8, 0x7D0, 0x5D2,
            0x3E8, 0x5DC, 0x5DC, 0x5DC, 0x5DC, 0x5DC, 0x5DC])));
        assert!(false);
    }
}
