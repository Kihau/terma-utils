use std::thread;
use terma_utils::{
    KeyCode,
    terma_init,
    read_key,
    console_clear,
    cursor_move,
    cursor_pos,
    color_fg,
    color_bg,
    color_reset,
};

fn console_test1() {
    loop {
        let value = read_key();
        color_bg(255, 0, 0);
        console_clear();
        cursor_move(30, 120);
        println!("{value:?}");

        let value = read_key();
        color_reset();
        console_clear();
        cursor_move(30, 120);
        println!("{value:?}");
    }
}

fn console_test2() {
    let mut x = 1i16;
    let mut y = 1i16;

    console_clear();
    loop {
        let value = read_key();
        match value {
            KeyCode::ArrowUp => x -= 1,
            KeyCode::ArrowDown => x += 1,
            KeyCode::ArrowRight => y += 1,
            KeyCode::ArrowLeft => y -= 1,
            _ => {}
        }
        // console_clear();
        // println!("{value:?}");
        cursor_move(x, y);
        print!("x");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        // thread::sleep_ms(2000);
    }
}

fn console_test3() {
    loop {
        println!("{:?}", read_key());
        cursor_pos();
    }
    // let _ = cursor_pos();
    // let _ = cursor_pos();
    // let _ = cursor_pos();
    // let _ = cursor_pos();
    // let _ = cursor_pos();
    // let _ = cursor_pos();
}

fn console_test4() { }

fn main() {
    terma_init();

    // console_test1();
    // console_test2();
    console_test3();
}
