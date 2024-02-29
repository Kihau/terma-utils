fn parse_pos(buffer: &[u8]) -> Pos {
    let mut i = 0;
    while i < buffer.len() {
        let byte = buffer[i];
        i += 1;

        if byte == 0 || byte == b'[' {
            break;
        }
    }

    let mut x = 0u16;
    while i < buffer.len() {
        let byte = buffer[i];
        i += 1;

        match byte {
            b'0'..=b'9' => {
                x *= 10;
                x += (byte - b'0') as u16;
            }
            _ => break,
        }
    }

    let mut y = 0u16;
    while i < buffer.len() {
        let byte = buffer[i];
        i += 1;

        match byte {
            b'0'..=b'9' => {
                y *= 10;
                y += (byte - b'0') as u16;
            }
            _ => break,
        }
    }

    return Pos { x, y }
}
