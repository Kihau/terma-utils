use std::io::Write;
use crate::{print_str, print_buf, read_buf, KeyCode, Pos};

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

pub(crate) fn cursor_set(x: u16, y: u16) {
    // ANSI console cursor position is 0 and NOT 1 indexed.
    let ansi_x = x.saturating_add(1);
    let ansi_y = y.saturating_add(1);

    let mut buffer = [0u8; 16];
    write!(&mut buffer[..], "\x1b[{ansi_y};{ansi_x}H").unwrap();
    print_buf(&buffer, buffer.len());
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

    return Pos { x, y }
}

pub(crate) fn cursor_get() -> Pos {
    let ansi_cursor_get = "\x1b[6n";
    print_str(ansi_cursor_get);

    let mut buffer = [0u8; 16];
    let _ = read_buf(&mut buffer);

    let mut pos = parse_pos(&buffer);
    pos.x = pos.x.saturating_sub(1);
    pos.y = pos.y.saturating_sub(1);

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
    let mut buffer = [0u8; 32];
    write!(&mut buffer[..], "\x1b[48;2;{red};{green};{blue}m").unwrap();
    print_buf(&buffer, buffer.len());
}

pub(crate) fn color_fg(red: u8, green: u8, blue: u8) {
    let mut buffer = [0u8; 32];
    write!(&mut buffer[..], "\x1b[38;2;{red};{green};{blue}m").unwrap();
    print_buf(&buffer, buffer.len());
}
