use crate::KeyCode;
use crate::Pos;
use crate::ansi;

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

pub fn print_str(string: &str) -> isize {
    unsafe {
        let bytes_written = write(
            STDOUT,
            string.as_ptr() as *const void, 
            string.len()
        );

        return bytes_written as isize;
    }
}

pub fn read_buf(buffer: &mut [u8]) -> isize {
    unsafe {
        let bytes_read = read(
            STDIN,
            buffer.as_mut_ptr() as *mut void,
            buffer.len()
        );

        return bytes_read as isize;
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

        let keycode = ansi::read_key();

        tcsetattr(STDIN, TCSANOW, &old_settings as *const Termios);
        return keycode;
    }
}

pub fn cursor_set(x: i16, y: i16) {
    ansi::cursor_set(x, y);
}

pub fn cursor_get() -> Pos {
    unsafe {
        let mut old_settings = Termios::default();
        tcgetattr(STDIN, &mut old_settings as *mut Termios);

        let mut new_settings = old_settings;
        new_settings.local_flags &= !(ICANON | ECHO);

        tcsetattr(STDIN, TCSANOW, &new_settings as *const Termios);

        flush_stdin();

        let pos = ansi::cursor_get();

        tcsetattr(STDIN, TCSANOW, &old_settings as *const Termios);
        return pos;
    }
}

pub fn console_clear() {
    ansi::console_clear();
}

pub fn color_reset() {
    ansi::color_reset();
}

pub fn color_bg(red: u8, green: u8, blue: u8) {
    ansi::color_bg(red, green, blue);
}

pub fn color_fg(red: u8, green: u8, blue: u8) {
    ansi::color_fg(red, green, blue);
}
