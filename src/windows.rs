use super::KeyCode;

#[allow(non_camel_case_types)]
type void = std::ffi::c_void;

/* WinAPI Types:
    BOOL  -> i32
    DWORD -> u32
    WORD  -> u16
    WCHAR -> u16
    CHAR  -> u8
*/

const STD_INPUT_HANDLE:  u32 = -10i32 as u32;
const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;

const ENABLE_LINE_INPUT: u32 = 0x0002;
const ENABLE_ECHO_INPUT: u32 = 0x0004;
const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x0200;
const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

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
    fn WriteConsoleW(handle: *const void, buffer: *const void, buffer_size: u32, bytes_written: *mut u32, reserved: *const void);

    fn GetConsoleScreenBufferInfo(handle: *const void, buffer_info: *mut ConsoleBufferInfo) -> i32;
    fn ScrollConsoleScreenBufferW(handle: *const void, scroll: *const SmallRect, clip: *const SmallRect, destination: Coord, fill: *const CharInfo) -> i32;
    fn SetConsoleCursorPosition(handle: *const void, cursor_position: Coord) -> i32;
}

unsafe fn fallback_read_key() -> KeyCode {
    use std::io::Read;

    let mut buffer = [0u8; 3];
    let _ = std::io::stdin().read(&mut buffer);
    return KeyCode::Char(char::from_u32_unchecked(buffer[0] as u32));
}

pub(crate) fn read_key() -> KeyCode {
    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        if handle == std::ptr::null() {
            return KeyCode::Error
        }

        let _ = FlushConsoleInputBuffer(handle);

        let mut entries_read = 0u32;
        let mut input = InputRecord::default();
        loop {
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
                    b'0'..=b'9' => return KeyCode::Char(char::from_u32_unchecked(data as u32)),
                    b'a'..=b'z' => return KeyCode::Char(char::from_u32_unchecked(data as u32)),
                    b'A'..=b'Z' => return KeyCode::Char(char::from_u32_unchecked(data as u32)),
                    8  => return KeyCode::Backspace,
                    13 => return KeyCode::Enter,
                    32 => return KeyCode::Space,
                    _  => return KeyCode::Other(data as u64),
                }
            }

            match key.virtual_keycode {
                0x25 => return KeyCode::ArrowLeft,
                0x26 => return KeyCode::ArrowUp,
                0x27 => return KeyCode::ArrowRight,
                0x28 => return KeyCode::ArrowDown,
                _    => continue,
            }
        }
    }
}

pub(crate) fn clear_console() {
    print!("\x1b[2J");

    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle == std::ptr::null() {
            return;
        }

        let mut buffer_info = ConsoleBufferInfo::default();
        GetConsoleScreenBufferInfo(handle, &mut buffer_info as *mut ConsoleBufferInfo);

        let rect = SmallRect {
            left: 0,
            top: 0,
            right: buffer_info.buffer_size.x,
            bottom: buffer_info.buffer_size.y,
        };

        let target = Coord {
            x: 0,
            y: 0 - buffer_info.buffer_size.y
        };

        let fill = CharInfo {
            unicode_char: 32,
            attributes: buffer_info.attributes,
        };

        ScrollConsoleScreenBufferW(handle, &rect as *const SmallRect, std::ptr::null(), target, &fill as *const CharInfo);

        buffer_info.cursor_position.x = 0;
        buffer_info.cursor_position.y = 0;
        SetConsoleCursorPosition(handle, buffer_info.cursor_position);
    }
}

pub(crate) fn move_cursor(x: u16, y: u16) {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle == std::ptr::null() {
            return;
        }

        let position = Coord { 
            x: x as i16,
            y: y as i16 
        };

        SetConsoleCursorPosition(handle, position);
    }
}
