use super::KeyCode;
use super::Pos;

const STDIN:  i32 = 0;
const STDOUT: i32 = 0;

#[allow(non_camel_case_types)]
type void = std::ffi::c_void;

const TCSANOW: i32 = 0;
const ICANON:  i32 = 2;
const ECHO:    i32 = 10;

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct Termios {
    input_flags:         i32,       // tcflag_t c_iflag;
    output_flags:        i32,       // tcflag_t c_oflag;
    control_flags:       i32,       // tcflag_t c_cflag;
    local_flags:         i32,       // tcflag_t c_lflag;
    line_discipline:     i32,       // cc_t     c_line;
    control_characters: [i32; 32],  // cc_t     c_cc[32];
    input_speed:         i32,       // speed_t  c_ispeed;
    output_speed:        i32,       // speed_t  c_ospeed;
}

const POLLIN: i16 = 1;

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct PollFd {
    file_descriptor: i32,  // int   fd
    request_events:  i16,  // short events
    return_events:   i16,  // short revents
}

extern "C" {
    fn tcgetattr(fd: i32, termios: *mut Termios) -> i32;
    fn tcsetattr(fd: i32, optional_actions: i32, termios: *const Termios) -> i32;
    fn read(fd: i32, buffer: *mut void, buffer_size: usize) -> i32;
    fn write(fd: i32, buffer: *const void, buffer_size: usize) -> i32;
    fn poll(fds: *mut PollFd, fds_count: u64, timeout: i32) -> i32;
    // fn setlocale(category: i32, locale: *const u8) -> *const u8;
}

pub fn terma_init() {

}

unsafe fn flush_stdin() {
    let mut pollfd = PollFd {
        file_descriptor: STDIN,
        request_events:  POLLIN,
        return_events:   0,
    };

    let mut poll_result = poll(&mut pollfd as *mut PollFd, 1, 0);
    if poll_result == -1 {
        return;
    }

    while pollfd.return_events != 0 {
        let mut buffer = [0u8; 1024];
        let _ = read(STDIN, buffer.as_mut_ptr() as *mut void, buffer.len());

        poll_result = poll(&mut pollfd as *mut PollFd, 1, 0);
        if poll_result == -1 {
            return;
        }
    }
}

pub fn print(string: &str) -> usize {
    unsafe {
        let bytes_written = write(
            STDOUT,
            string.as_ptr() as *const void, 
            string.len()
        );

        return bytes_written as usize;
    }
}

pub fn read_key() -> KeyCode {
    unsafe {
        let mut old_settings = Termios::default();
        tcgetattr(STDIN, &mut old_settings as *mut Termios);

        let mut new_settings = old_settings;
        new_settings.local_flags &= !(ICANON | ECHO);

        tcsetattr(STDIN, TCSANOW, &new_settings as *const Termios);

        flush_stdin();

        let keycode = loop {
            let mut buffer = [0u8; 8];
            let _ = read(STDIN, buffer.as_mut_ptr() as *mut void, 8);

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

            // print!("{:?}", String::from_utf8_lossy(&buffer));
            let data = buffer[0] as u8;
            match data {
                b'0'..=b'9' => break KeyCode::Char(char::from_u32_unchecked(data as u32)),
                b'a'..=b'z' => break KeyCode::Char(char::from_u32_unchecked(data as u32)),
                b'A'..=b'Z' => break KeyCode::Char(char::from_u32_unchecked(data as u32)),
                10  => break KeyCode::Enter,
                32  => break KeyCode::Space,
                127 => break KeyCode::Backspace,
                _   => {}
                // _   => break KeyCode::Other(u64::from_ne_bytes(buffer)),
            }
        };

        tcsetattr(STDIN, TCSANOW, &old_settings as *const Termios);
        return keycode;
    }
}

pub fn cursor_set(x: i16, y: i16) {
    unsafe {
        //                    VVVVVVVV Can be changed to something like a 64 byte buffer to avoid useless allocations.
        let ansi_cursor_set = format!("\x1b[{x};{y}H");
        write(STDOUT, ansi_cursor_set.as_str().as_ptr() as *const void, ansi_cursor_set.len());
    }
}

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

pub fn cursor_get() -> Pos {
    unsafe {
        let mut old_settings = Termios::default();
        tcgetattr(STDIN, &mut old_settings as *mut Termios);

        let mut new_settings = old_settings;
        new_settings.local_flags &= !(ICANON | ECHO);

        tcsetattr(STDIN, TCSANOW, &new_settings as *const Termios);

        flush_stdin();

        let ansi_cursor_get = "\x1b[6n";
        write(STDOUT, ansi_cursor_get.as_ptr() as *const void, ansi_cursor_get.len());

        let mut buffer = [0u8; 16];
        let _ = read(STDIN, buffer.as_mut_ptr() as *mut void, 16);

        let pos = parse_pos(&buffer);

        tcsetattr(STDIN, TCSANOW, &old_settings as *const Termios);

        return pos;
    }
}

pub fn console_clear() {
    unsafe {
        let ansi_move = "\x1b[1;1H";
        write(STDOUT, ansi_move.as_ptr() as *const void, ansi_move.len());

        let ansi_clear = "\x1b[0J";
        write(STDOUT, ansi_clear.as_ptr() as *const void, ansi_clear.len());
    }
}

pub fn color_reset() {
    unsafe {
        let ansi_reset = "\x1b[0m";
        write(STDOUT, ansi_reset.as_ptr() as *const void, ansi_reset.len());
    }
}

pub fn color_bg(red: u8, green: u8, blue: u8) {
    unsafe {
        let ansi_color_bg = format!("\x1b[48;2;{red};{green};{blue}m");
        write(STDOUT, ansi_color_bg.as_str().as_ptr() as *const void, ansi_color_bg.len());
    }
}

pub fn color_fg(red: u8, green: u8, blue: u8) {
    unsafe {
        //                  VVVVVVV Can be changed to something like a 64 byte buffer to avoid useless allocations.
        let ansi_color_bg = format!("\x1b[38;2;{red};{green};{blue}m");
        write(STDOUT, ansi_color_bg.as_str().as_ptr() as *const void, ansi_color_bg.len());
    }
}
