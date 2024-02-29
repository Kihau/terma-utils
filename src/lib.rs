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

#[cfg(all(unix))]
pub use unix::{
    terma_init,
    read_key,
    console_clear,
    cursor_move,
    cursor_pos,
    color_bg,
    color_fg,
    color_reset,
};

#[cfg(target_os = "windows")]
pub use windows::{
    terma_init,
    read_key,
    console_clear,
    move_cursor,
    color_bg,
    color_fg,
};
