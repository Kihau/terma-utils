use super::KeyCode;
use super::Pos;
use super::ansi;

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
static mut stdin:  *const void = std::ptr::null();
static mut stdout: *const void = std::ptr::null();

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
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
        // TODO: perform checks here:
        // print ansi query code
        // check if query returned something
        // if yes, set virtual supported to true
        // if not, set virtual supported to false and clear output buffer
        //
        // no ansi -> WriteConsole buffers output(?), set correct console modes for printing
        // ansi -> enable virtual processing
        supports_ansi = false;

        // Return error on null?
        stdin = GetStdHandle(STD_INPUT_HANDLE);
        if stdin != std::ptr::null() && supports_ansi {
            let mut input_mode = 0;
            input_mode |= ENABLE_PROCESSED_INPUT;
            input_mode |= ENABLE_VIRTUAL_TERMINAL_INPUT;
            // input_mode |= ENABLE_ECHO_INPUT;
            // input_mode |= ENABLE_LINE_INPUT;
            SetConsoleMode(stdin, input_mode);
        }

        stdout = GetStdHandle(STD_OUTPUT_HANDLE);
        if stdout != std::ptr::null() && supports_ansi {
            let mut output_mode = 0;
            output_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
            output_mode |= ENABLE_WRAP_AT_EOL_OUTPUT;
            output_mode |= ENABLE_PROCESSED_OUTPUT;
            SetConsoleMode(stdout, output_mode);
        }
    }
}

unsafe fn fallback_read_key() -> KeyCode {
    use std::io::Read;

    let mut buffer = [0u8; 3];
    let _ = std::io::stdin().read(&mut buffer);
    return KeyCode::Char(char::from_u32_unchecked(buffer[0] as u32));
}

// NOTE: Works very poorly on mingw and git bash terminals.
pub fn read_key() -> KeyCode {
    unsafe { 
        let _ = FlushConsoleInputBuffer(stdin); 

        if supports_ansi {
            return ansi::read_key();
        } else {
            return read_key_legacy();
        }
    }
}

unsafe fn read_key_legacy() -> KeyCode {
    let mut entries_read = 0u32;
    let mut input = InputRecord::default();
    let key = loop {
        let result = ReadConsoleInputW(stdin, &mut input as *mut InputRecord, 1, &mut entries_read as *mut u32);

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

pub fn print_str(string: &str) -> isize {
    unsafe {
        let mut bytes_written: u32 = 0;
        let result = WriteConsoleA(
            stdout,
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
        let mut bytes_read = 0u32;
        let result = ReadConsoleA(
            stdin,
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
        if supports_ansi {
            ansi::console_clear();
        } else {
            console_clear_legacy();
        }
    }
}

unsafe fn console_clear_legacy() {
    let mut buffer_info = ConsoleBufferInfo::default();
    let result = GetConsoleScreenBufferInfo(stdout, &mut buffer_info as *mut ConsoleBufferInfo);
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

    let result = ScrollConsoleScreenBufferW(stdout, &rect as *const SmallRect, std::ptr::null(), target, &fill as *const CharInfo);
    if result == 0 {
        return;
    }

    buffer_info.cursor_position.x = 0;
    buffer_info.cursor_position.y = 0;
    SetConsoleCursorPosition(stdout, buffer_info.cursor_position);
}

pub fn cursor_get() -> Pos {
    unsafe {
        if supports_ansi {
            return ansi::cursor_get();
        } else {
            return cursor_get_legacy();
        }
    }
}

unsafe fn cursor_get_legacy() -> Pos {
    let mut buffer_info = ConsoleBufferInfo::default();
    let _ = GetConsoleScreenBufferInfo(stdout, &mut buffer_info as *mut ConsoleBufferInfo);

    let pos = Pos {
        x: buffer_info.cursor_position.x as u16,
        y: buffer_info.cursor_position.y as u16,
    };

    return pos;
}

pub fn cursor_set(x: i16, y: i16) {
    unsafe {
        if supports_ansi {
            return ansi::cursor_set(x, y);
        } else {
            return cursor_set_legacy(x, y);
        }
    }
}

unsafe fn cursor_set_legacy(x: i16, y: i16) {
    let position = Coord { x: x - 1, y: y - 1 };
    SetConsoleCursorPosition(stdout, position);
}

pub fn color_bg(red: u8, green: u8, blue: u8) {
    unsafe {
        if supports_ansi {
            ansi::color_bg(red, green, blue);
        } else {
            // Not supported. Fallback to legacy windows console colors?
        }
    }
}

pub fn color_fg(red: u8, green: u8, blue: u8) {
    unsafe {
        if supports_ansi {
            ansi::color_fg(red, green, blue);
        } else {
            // Not supported. Fallback to legacy windows console colors?
        }
    }
}

pub fn color_reset() {
    unsafe {
        if supports_ansi {
            ansi::color_reset();
        } else {
            color_reset_legacy();
        }
    }
}

unsafe fn color_reset_legacy() {
    let color_white = 15;
    SetConsoleTextAttribute(stdout, color_white);
}
