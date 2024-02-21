use super::KeyCode;

const STDIN:   i32 = 0;

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
    fn read(fd: i32, buffer: *mut void, buffer_size: i32) -> i32;
    fn poll(fds: *mut PollFd, fds_count: u64, timeout: i32) -> i32;
    // fn setlocale(category: i32, locale: *const u8) -> *const u8;
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
        let _ = read(STDIN, buffer.as_mut_ptr() as *mut void, buffer.len() as i32);

        poll_result = poll(&mut pollfd as *mut PollFd, 1, 0);
        if poll_result == -1 {
            return;
        }
    }
}

pub(crate) fn read_key() -> KeyCode {
    unsafe {
        let mut old_settings = Termios::default();
        tcgetattr(STDIN, &mut old_settings as *mut Termios);

        let mut new_settings = old_settings;
        new_settings.local_flags &= !(ICANON | ECHO);

        tcsetattr(STDIN, TCSANOW, &new_settings as *const Termios);
        flush_stdin();

        let mut buffer = [0u8; 8];
        let _ = read(STDIN, buffer.as_mut_ptr() as *mut void, 8);

        tcsetattr(STDIN, TCSANOW, &old_settings as *const Termios);

        if buffer[0] == 0x1b && buffer[1] == b'[' {
            let next = if buffer[2] == 0x31 && buffer[3] == 0x3b {
                5
            } else {
                2
            };

            match buffer[next] {
                65 => return KeyCode::ArrowUp,
                66 => return KeyCode::ArrowDown,
                67 => return KeyCode::ArrowRight,
                68 => return KeyCode::ArrowLeft,
                _ => {}
            }
        }

        let data = buffer[0] as u8;
        match data {
            b'0'..=b'9' => return KeyCode::Char(char::from_u32_unchecked(data as u32)),
            b'a'..=b'z' => return KeyCode::Char(char::from_u32_unchecked(data as u32)),
            b'A'..=b'Z' => return KeyCode::Char(char::from_u32_unchecked(data as u32)),
            10  => return KeyCode::Enter,
            32  => return KeyCode::Space,
            127 => return KeyCode::Backspace,
            _   => return KeyCode::Other(u64::from_ne_bytes(buffer)),
        }
    }
}

const ESC:   &'static str = "\x1b";
const CSI:   &'static str = "\x1b[";
const CLEAR: &'static str = "\x1b[0J";
const MOVE:  &'static str = "\x1b[1;1H";

pub(crate) fn move_cursor(x: u16, y: u16) {
    print!("{CSI}{x};{y}H");
}

pub(crate) fn clear_console() {
    move_cursor(1, 1);
    print!("{CSI}0J");
}
