#[derive(Debug)]
pub struct Pos { 
    pub x: u16, 
    pub y: u16,
}

#[derive(Debug)]
pub enum KeyCode {
    // TODO(?): 
    //     Add modifiers (Ctrl, Alt) + console events (buffer resize, mouse click)?
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

mod ansi;

// TODO: 
//   - try_read_key() - Non-blocking read_key
//   - console_get()  - Get info about console (?)
//   - clear_line()   - Clear line at current cursor position
//   - clear_end()    - Clear console at current cursor position till end 

#[cfg(all(unix))]
pub use unix::{
    terma_init,
    print_str,
    print_buf,
    read_buf,
    read_key,
    console_clear,
    cursor_get,
    cursor_set,
    color_bg,
    color_fg,
    color_reset,
    buffer_size,
};

#[cfg(target_os = "windows")]
pub use windows::{
    terma_init,
    print_str,
    print_buf,
    read_buf,
    read_key,
    console_clear,
    cursor_get,
    cursor_set,
    color_bg,
    color_fg,
    color_reset,
    buffer_size,
};
