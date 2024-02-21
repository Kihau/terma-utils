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

// ffi pointer convertion macro:
// ptr!(variable, Type)     -> &variable as *const VariableType as *const Type;
// ptr_mut!(variable, Type) -> &mut variable as *mut VariableType as *mut Type;

#[cfg(all(unix))]
pub mod unix;

#[cfg(target_os = "windows")]
pub mod windows;

pub fn read_key() -> KeyCode {
    #[cfg(target_os = "windows")]
    return windows::read_key();

    #[cfg(all(unix))]
    return unix::read_key();
}

pub fn clear_console() {
    #[cfg(target_os = "windows")]
    return windows::clear_console();

    #[cfg(all(unix))]
    return unix::clear_console();
}

pub fn move_cursor(x: u16, y: u16) {
    #[cfg(target_os = "windows")]
    return windows::move_cursor(y, x);

    #[cfg(all(unix))]
    return unix::move_cursor(x, y);
}
