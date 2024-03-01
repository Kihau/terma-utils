use super::KeyCode;
use super::Pos;

#[allow(non_camel_case_types)]
type void = std::ffi::c_void;

/* WinAPI Types:
    BOOL  -> i32
    DWORD -> u32
    WORD  -> u16
    WCHAR -> u16
    CHAR  -> u8
*/

// Console handle codes.
const STD_INPUT_HANDLE:  u32 = -10i32 as u32;
const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;

// Input flags.
const ENABLE_PROCESSED_INPUT: u32        = 0x0001;
const ENABLE_LINE_INPUT: u32             = 0x0002;
const ENABLE_ECHO_INPUT: u32             = 0x0004;
const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x0200;

// Output flags.
const ENABLE_PROCESSED_OUTPUT: u32            = 0x0001;
const ENABLE_WRAP_AT_EOL_OUTPUT: u32          = 0x0002;
const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

// NOTE: Only modified during the initialization (terma_init).
//       The initialization should happen before any thread is spawned.
static mut supports_ansi: bool = false;

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct Coord {
    x: i16,  // SHORT X;
    y: i16,  // SHORT Y;
}

#[repr(C)]
#[derive(Copy, Clone)]
struct EventKey {
    key_down:          i32,   // BOOL  bKeyDown;
    repeat_count:      u16,   // WORD  wRepeatCount;
    virtual_keycode:   u16,   // WORD  wVirtualKeyCode;
    virtual_scancode:  u16,   // WORD  wVirtualScanCode;
    //character_data:  u8,    // CHAR  AsciiChar;
    character_data:    u16,   // WCHAR UnicodeChar;
    control_key_state: u32,   // DWORD dwControlKeyState;
}

#[repr(C)]
#[derive(Copy, Clone)]
struct EventMouse {
    mouse_position:     Coord,  // COORD dwMousePosition;
    button_state:       u32,    // DWORD dwButtonState;
    control_key_state:  u32,    // DWORD dwControlKeyState;
    event_flags:        u32,    // DWORD dwEventFlags;
}

#[repr(C)]
#[derive(Copy, Clone)]
union Event {
    key:   EventKey,    // KEY_EVENT_RECORD          KeyEvent;
    mouse: EventMouse,  // MOUSE_EVENT_RECORD        MouseEvent;
    size:  Coord,       // WINDOW_BUFFER_SIZE_RECORD WindowBufferSizeEvent;
    menu:  u32,         // MENU_EVENT_RECORD         MenuEvent;
    focus: i32,         // FOCUS_EVENT_RECORD        FocusEvent;
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InputRecord {
    event_type: u16,  // WORD  EventType;
    event: Event,     // union { ... } Event;
}

impl Default for InputRecord {
    fn default() -> Self {
        Self {
            event_type: 0,
            event: Event { focus: 0 }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct CharInfo {
    //ascii_char: u8,   // CHAR  AsciiChar;
    unicode_char: u16,  // WCHAR UnicodeChar;
    attributes:   u16,  // WORD  Attributes;
} 

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct SmallRect {
    left:   i16,  // SHORT Left;
    top:    i16,  // SHORT Top;
    right:  i16,  // SHORT Right;
    bottom: i16,  // SHORT Bottom;
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct ConsoleBufferInfo {
    buffer_size:     Coord,      // COORD      dwSize;
    cursor_position: Coord,      // COORD      dwCursorPosition;
    attributes:      u16,        // WORD       wAttributes;
    window_coords:   SmallRect,  // SMALL_RECT srWindow;
    maximum_size:    Coord,      // COORD      dwMaximumWindowSize;
}

extern "system" {
    fn GetStdHandle(std_handle_code: u32) -> *const void;
    fn FlushConsoleInputBuffer(handle: *const void) -> i32;
    fn ReadConsoleInputW(handle: *const void, buffer: *mut InputRecord, buffer_length: i32, entries_read: *mut u32) -> i32;
    fn GetConsoleMode(handle: *const void, mode: *mut u32) -> i32;
    fn SetConsoleMode(handle: *const void, mode: u32) -> i32;
    fn ReadConsoleA(handle: *const void, buffer: *mut void, buffer_size: u32, bytes_read: *mut u32, input_control: *const u32) -> i32;
    fn WriteConsoleW(handle: *const void, buffer: *const void, buffer_size: u32, bytes_written: *mut u32, reserved: *const void) -> i32;
    fn WriteConsoleA(handle: *const void, buffer: *const void, buffer_size: u32, bytes_written: *mut u32, reserved: *const void) -> i32;

    fn GetConsoleScreenBufferInfo(handle: *const void, buffer_info: *mut ConsoleBufferInfo) -> i32;
    fn ScrollConsoleScreenBufferW(handle: *const void, scroll: *const SmallRect, clip: *const SmallRect, destination: Coord, fill: *const CharInfo) -> i32;
    fn SetConsoleCursorPosition(handle: *const void, cursor_position: Coord) -> i32;
    fn SetConsoleTextAttribute(handle: *const void, attributes: u16) -> i32;
}

pub fn terma_init() {
    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        if handle != std::ptr::null() {
            let mut input_mode = 0;
            input_mode |= ENABLE_PROCESSED_INPUT;
            input_mode |= ENABLE_VIRTUAL_TERMINAL_INPUT;
            // input_mode |= ENABLE_ECHO_INPUT;
            // input_mode |= ENABLE_LINE_INPUT;
            SetConsoleMode(handle, input_mode);
        }

        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle != std::ptr::null() {
            let mut output_mode = 0;
            output_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
            output_mode |= ENABLE_WRAP_AT_EOL_OUTPUT;
            output_mode |= ENABLE_PROCESSED_OUTPUT;
            SetConsoleMode(handle, output_mode);
        }

        // TODO: perform checks here:
        // print ansi query code
        // check if query returned something
        // if yes, set virtual supported to true
        // if not, set virtual supported to false and clear output buffer
        //
        // no ansi -> WriteConsole buffers output(?), set correct console modes for printing
        // ansi -> enable virtual processing
        supports_ansi = true;
    }
}

unsafe fn fallback_read_key() -> KeyCode {
    use std::io::Read;

    let mut buffer = [0u8; 3];
    let _ = std::io::stdin().read(&mut buffer);
    return KeyCode::Char(char::from_u32_unchecked(buffer[0] as u32));
}

pub fn read_key() -> KeyCode {
    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        if handle == std::ptr::null() {
            return KeyCode::Error
        }

        let _ = FlushConsoleInputBuffer(handle);

        let keycode = loop {
            let mut buffer = [0u8; 8];
            let mut bytes_read = 0u32;
            let _ = ReadConsoleA(handle, buffer.as_mut_ptr() as *mut void, 8, &mut bytes_read as *mut u32, std::ptr::null());

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

        return keycode;
        // let mut buffer = [0u8; 8];
        // let mut bytes_read = 0u32;
        // let _ = ReadConsoleA(handle, buffer.as_mut_ptr() as *mut void, buffer.len() as u32, &mut bytes_read as *mut u32, std::ptr::null());
        // println!("{buffer:?}");
        // return KeyCode::Other(0);
    }

}

// NOTE: Works very poorly on mingw and git bash terminals.
// TODO(?): ANSI version of read_key for windows?
pub fn read_key_legacy() -> KeyCode {
    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        if handle == std::ptr::null() {
            return KeyCode::Error
        }

        let _ = FlushConsoleInputBuffer(handle);

        let mut entries_read = 0u32;
        let mut input = InputRecord::default();
        let key = loop {
            let result = ReadConsoleInputW(handle, &mut input as *mut InputRecord, 1, &mut entries_read as *mut u32);

            // Reading the console input failed.
            if result == 0 || entries_read == 0 {
                return fallback_read_key();
            }

            if input.event_type != 1 {
                continue;
            }

            let key = input.event.key;
            if key.key_down == 0 {
                continue;
            }

            let key = input.event.key;

            if key.character_data != 0 {
                let data = key.character_data;
                match key.character_data as u8 {
                    b'0'..=b'9' => break KeyCode::Char(char::from_u32_unchecked(data as u32)),
                    b'a'..=b'z' => break KeyCode::Char(char::from_u32_unchecked(data as u32)),
                    b'A'..=b'Z' => break KeyCode::Char(char::from_u32_unchecked(data as u32)),
                    8  => break KeyCode::Backspace,
                    13 => break KeyCode::Enter,
                    32 => break KeyCode::Space,
                    _  => {}
                    // _  => break KeyCode::Other(data as u64),
                }
            }

            match key.virtual_keycode {
                0x25 => break KeyCode::ArrowLeft,
                0x26 => break KeyCode::ArrowUp,
                0x27 => break KeyCode::ArrowRight,
                0x28 => break KeyCode::ArrowDown,
                _    => continue,
            }
        };

        return key;
    }
}

pub fn print_str(string: &str) -> isize {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle == std::ptr::null() {
            return -1;
        }

        let mut bytes_written: u32 = 0;
        let result = WriteConsoleA(
            handle,
            string.as_ptr() as *const void,
            string.len() as u32,
            &mut bytes_written as *mut u32,
            std::ptr::null()
        );

        if result == 0 {
            return -1;
        }

        return bytes_written as isize;
    }
}

pub fn read_buf(buffer: &mut [u8]) -> isize {
    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        if handle == std::ptr::null() {
            return -1;
        }

        let mut bytes_read = 0u32;
        let result = ReadConsoleA(
            handle,
            buffer.as_mut_ptr() as *mut void,
            buffer.len() as u32,
            &mut bytes_read as *mut u32,
            std::ptr::null()
        );

        if result == 0 {
            return -1;
        }

        return bytes_read as isize;
    }
}

pub fn console_clear() {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle == std::ptr::null() {
            return;
        }

        if supports_ansi {
            let ansi_move = "\x1b[1;1H";
            let _ = print_str(ansi_move);

            let ansi_clear = "\x1b[0J";
            let _ = print_str(ansi_clear);
        } else {
            let mut buffer_info = ConsoleBufferInfo::default();
            let result = GetConsoleScreenBufferInfo(handle, &mut buffer_info as *mut ConsoleBufferInfo);
            if result == 0 {
                return;
            }

            let rect = SmallRect {
                left:   0,
                top:    0,
                right:  buffer_info.buffer_size.x,
                bottom: buffer_info.buffer_size.y,
            };

            let target = Coord {
                x: 0,
                y: 0 - buffer_info.buffer_size.y
            };

            let fill = CharInfo {
                unicode_char: 32,
                attributes:   buffer_info.attributes,
            };

            let result = ScrollConsoleScreenBufferW(handle, &rect as *const SmallRect, std::ptr::null(), target, &fill as *const CharInfo);
            if result == 0 {
                return;
            }

            buffer_info.cursor_position.x = 0;
            buffer_info.cursor_position.y = 0;
            SetConsoleCursorPosition(handle, buffer_info.cursor_position);
        }
    }
}

pub fn cursor_get() -> Pos {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle == std::ptr::null() {
            return Pos { x: 0, y: 0 };
        }

        if supports_ansi {
            let ansi_cursor_get = "\x1b[6n";
            print_str(ansi_cursor_get);

            let handle = GetStdHandle(STD_INPUT_HANDLE);
            if handle == std::ptr::null() {
                return Pos { x: 0, y: 0 };
            }

            let mut buffer = [0u8; 64];
            let mut bytes_read = 0u32;
            ReadConsoleA(handle, buffer.as_mut_ptr() as *mut void, 64, &mut bytes_read as *mut u32, std::ptr::null());
            //println!("{buffer:?}");
            return Pos { x: 0, y: 0 };
        } else {
            let mut buffer_info = ConsoleBufferInfo::default();
            let _ = GetConsoleScreenBufferInfo(handle, &mut buffer_info as *mut ConsoleBufferInfo);

            let pos = Pos {
                x: buffer_info.cursor_position.x as u16,
                y: buffer_info.cursor_position.y as u16,
            };

            return pos;
        }
    }
}

pub fn cursor_set(x: i16, y: i16) {
    unsafe {
        if supports_ansi {
            let ansi_cursor_set = format!("\x1b[{x};{y}H");
            let _ = print_str(&ansi_cursor_set);
        } else {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if handle == std::ptr::null() {
                return;
            }

            let position = Coord { x: x - 1, y: y - 1 };
            SetConsoleCursorPosition(handle, position);
        }
    }
}

pub fn color_bg(red: u8, green: u8, blue: u8) {
    unsafe {
        if supports_ansi {
            let ansi_color_bg = format!("\x1b[48;2;{red};{green};{blue}m");
            print_str(&ansi_color_bg);
        } else {
            // Not supported. Fallback to legacy windows console colors?
        }
    }
}

pub fn color_fg(red: u8, green: u8, blue: u8) {
    unsafe {
        if supports_ansi {
            let ansi_color_fg = format!("\x1b[38;2;{red};{green};{blue}m");
            print_str(&ansi_color_fg);
        } else {
            // Not supported. Fallback to legacy windows console colors?
        }
    }
}

pub fn color_reset() {
    unsafe {
        if supports_ansi {
            let ansi_reset = "\x1b[0m";
            print_str(ansi_reset);
        } else {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if handle == std::ptr::null() {
                return;
            }
            let color_white = 15;
            SetConsoleTextAttribute(handle, color_white);
        }
    }
}
