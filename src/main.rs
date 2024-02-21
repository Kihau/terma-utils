use std::thread;

use terma_utils::{KeyCode, read_key, clear_console, move_cursor};

fn moving_x() {
    let mut x = 1;
    let mut y = 1;

    clear_console();
    loop {
        let value = read_key();
        match value {
            KeyCode::ArrowUp => x -= 1,
            KeyCode::ArrowDown => x += 1,
            KeyCode::ArrowRight => y += 1,
            KeyCode::ArrowLeft => y -= 1,
            _ => {}
        }
        // clear_console();
        // println!("{value:?}");
        move_cursor(x, y);
        print!("x");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        // thread::sleep_ms(2000);
    }
}

fn console_test() {
    loop {
        let value = read_key();
        clear_console();
        println!("{value:?}");
        // thread::sleep_ms(2000);
    }
}

fn main() {
    // let _ = std::process::Command::new("sh")
    //     .args(["-c", "clear"])
    //     .spawn();

    // let _ = std::process::Command::new("clear").spawn();

    println!("Hello, world!");
    // thread::sleep_ms(2000);

    // console_test();
    moving_x();
}
