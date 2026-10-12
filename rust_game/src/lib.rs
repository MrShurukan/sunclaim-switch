#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

mod runtime;
mod raylib;
mod game;
mod panic_buffer;

use alloc::ffi::CString;
use alloc::format;
use core::ffi::c_int;

use crate::game::input::GameInput;
use crate::game::{Game, GameError};
use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::raylib::Gamepad;
use raylib::{
    App,
    Color,
};

#[no_mangle]
pub extern "C" fn rust_main() -> c_int {
    let Ok(mut app) =
        App::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "Rust Switch",
        )
    else {
        return 1;
    };

    app.set_target_fps(60);

    let mut game = match Game::new() {
        Ok(game) => game,
        Err(e) => game_construction_error_loop(app, e),
    };

    let gamepad = Gamepad::new();

    while app.running() {
        let dt = app.delta_time();

        let input = GameInput::read(&gamepad);
        game.update(&input, dt);

        let mut frame =
            app.begin_frame(Color::rgb(10, 10, 10));
        game.draw(&mut frame);

        // EndDrawing() happens automatically here.
    }

    // CloseWindow() automatically happens
    // when App is dropped.

    0
}

fn game_construction_error_loop(mut app: App, err: GameError) -> ! {
    let c_string = CString::new(format!("{}", err)).expect("CString::new failed");
    let error_text = c_string.as_c_str();

    loop {
        let mut frame = app.begin_frame(Color::PANIC_RED);

        frame.const_text(
            c"Startup failure!",
            (40, 40).into(),
            48,
            Color::RED
        );

        frame.const_text(
            c"Game::new() failed to initialize.",
            (40, 120).into(),
            22,
            Color::WHITE
        );

        frame.const_text(
            error_text,
            (40, 160).into(),
            22,
            Color::WHITE
        );
    }
}
