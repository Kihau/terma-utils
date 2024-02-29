#[derive(Debug)]
pub struct Pos {
    pub x: u16, 
    pub y: u16,
}

#[derive(Debug)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Space,
    ArrowUp,
    ArrowDown,
    ArrowRight,
    ArrowLeft,
    Other(u64),
    Error,
}

#[cfg(all(unix))]
mod unix;

#[cfg(target_os = "windows")]
mod windows;


// TODO: 
//   - try_read_key() - Non-blocking read_key

#[cfg(all(unix))]
pub use unix::{
    terma_init,
    print_str,
    read_key,
    console_clear,
    cursor_get,
    cursor_set,
    color_bg,
    color_fg,
    color_reset,
};

#[cfg(target_os = "windows")]
pub use windows::{
    terma_init,
    print_str,
    read_key,
    console_clear,
    cursor_get,
    cursor_set,
    color_bg,
    color_fg,
    color_reset,
};
