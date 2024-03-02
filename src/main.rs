use std::thread;
use terma_utils::{
    KeyCode,
    terma_init,
    read_key,
    console_clear,
    cursor_get,
    cursor_set,
    color_fg,
    color_bg,
    color_reset,
};

fn console_test1() {
    loop {
        let value = read_key();
        color_bg(255, 0, 0);
        console_clear();
        cursor_set(30, 120);
        println!("{value:?}");

        let value = read_key();
        color_reset();
        console_clear();
        cursor_set(30, 120);
        println!("{value:?}");
    }
}

fn console_test2() {
    let mut x = 10i16;
    let mut y = 10i16;

    //console_clear();
    loop {
        let value = read_key();
        //println!("{value:?}");
        match value {
            KeyCode::ArrowUp => y -= 1,
            KeyCode::ArrowDown => y += 1,
            KeyCode::ArrowRight => x += 1,
            KeyCode::ArrowLeft => x -= 1,
            _ => {}
        }

        use std::io::Write;

        console_clear();

        cursor_set(x, y - 1);
        print!("   TERMA UTILS");
        std::io::stdout().flush();

        cursor_set(x, y);
        print!("{:?}", cursor_get());
        std::io::stdout().flush();

        cursor_set(x, y + 1);
        print!("    {:?}", value);
        std::io::stdout().flush();

        cursor_set(x + 8, y + 2);
    }
}

fn console_test3() {
    loop {
        let key = read_key();
        println!("{key:?}");
        // let pos = cursor_get();
        // println!("{pos:?}");
        // use std::io::Read;
        // let mut stdin = std::io::stdin();
        // let mut buffer = Vec::new();
        // let _ = stdin.read(&mut buffer);
    }
}

fn console_test4() { }

fn main() {
    terma_init();

    // console_test1();
    console_test2();
    // console_test3();
}
