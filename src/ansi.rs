use crate::{print_str, read_buf, KeyCode, Pos};

pub(crate) fn read_key() -> KeyCode {
    let keycode = loop {
        let mut buffer = [0u8; 8];
        let _ = read_buf(&mut buffer);

        if buffer[0] == 0x1b && buffer[1] == b'[' {
            let next = if buffer[2] == 0x31 && buffer[3] == 0x3b {
                5
            } else {
                2
            };

            match buffer[next] {
                65 => break KeyCode::ArrowUp,
                66 => break KeyCode::ArrowDown,
                67 => break KeyCode::ArrowRight,
                68 => break KeyCode::ArrowLeft,
                _ => {}
            }
        }

        let data = buffer[0] as u8;
        match data {
            b'0'..=b'9' => break KeyCode::Char(char::from_u32(data as u32).unwrap()),
            b'a'..=b'z' => break KeyCode::Char(char::from_u32(data as u32).unwrap()),
            b'A'..=b'Z' => break KeyCode::Char(char::from_u32(data as u32).unwrap()),
            b'\n'  => break KeyCode::Enter,
            b'\r'  => break KeyCode::Enter,
            b' '   => break KeyCode::Space,
            127 => break KeyCode::Backspace,
            _   => {}
            // _   => break KeyCode::Other(u64::from_ne_bytes(buffer)),
        }
    };

    return keycode;
}

pub(crate) fn cursor_set(x: i16, y: i16) {
    //                    VVVVVVVV Can be changed to something like a 64 byte buffer to avoid useless allocations.
    let ansi_cursor_set = format!("\x1b[{y};{x}H");
    print_str(&ansi_cursor_set);
}

pub(crate) fn parse_pos(buffer: &[u8]) -> Pos {
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

pub(crate) fn cursor_get() -> Pos {
    let ansi_cursor_get = "\x1b[6n";
    print_str(ansi_cursor_get);

    let mut buffer = [0u8; 16];
    let _ = read_buf(&mut buffer);

    let pos = parse_pos(&buffer);
    return pos;
}

pub(crate) fn console_clear() {
    let ansi_move = "\x1b[1;1H";
    print_str(ansi_move);

    let ansi_clear = "\x1b[0J";
    print_str(ansi_clear);
}

pub(crate) fn color_reset() {
    let ansi_reset = "\x1b[0m";
    print_str(ansi_reset);
}

pub(crate) fn color_bg(red: u8, green: u8, blue: u8) {
    let ansi_color_bg = format!("\x1b[48;2;{red};{green};{blue}m");
    print_str(ansi_color_bg.as_str());
}

pub(crate) fn color_fg(red: u8, green: u8, blue: u8) {
    let ansi_color_fg = format!("\x1b[38;2;{red};{green};{blue}m");
    print_str(ansi_color_fg.as_str());
}
