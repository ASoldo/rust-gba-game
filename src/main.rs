// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't really have an operating
// system, so most of the content of the standard library doesn't apply.
//
// Provided you haven't disabled it, agb does provide an allocator, so it is possible
// to use both the `core` and the `alloc` built in crates.
#![no_std]
// `agb` defines its own `main` function, so you must declare your game's main function
// using the #[agb::entry] proc macro. Failing to do so will cause failure in linking
// which won't be a particularly clear error message.
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    display::object::{Graphics, Tag},
    include_aseprite,
    input::Button,
    println, Gba,
};

const GRAPHICS: &Graphics = include_aseprite!("assets/Sprites.aseprite");
const SPRITE1: &Tag = &GRAPHICS.tags().get("Sprite1");
const SPRITE2: &Tag = &GRAPHICS.tags().get("Sprite2");

struct Position {
    x: i32,
    y: i32,
}
struct DemoLog {
    pub demo_id: i32,
}

// The main function must take 1 arguments and never return. The agb::entry decorator
// ensures that everything is in order. `agb` will call this after setting up the stack
// and interrupt handlers correctly. It will also handle creating the `Gba` struct for you.
#[cfg_attr(feature = "entry", agb::entry)]
fn main(mut gba: Gba) -> ! {
    println!("Demo Game");
    let soldo: i32 = 10;
    println!("{:?}", soldo);
    let so: DemoLog = DemoLog { demo_id: 1 };
    println!("{:?}", so.demo_id);

    // agb::no_game(gba);

    let mut input = agb::input::ButtonController::new();
    let object = gba.display.object.get_managed();
    let mut sprite1 = object.object_sprite(SPRITE1.sprite(0));
    let mut sprite1_pos = Position { x: 50, y: 50 };
    sprite1
        .set_x(sprite1_pos.x as u16)
        .set_y(sprite1_pos.y as u16)
        .show();

    let mut sprite2 = object.object_sprite(SPRITE2.sprite(0));
    sprite2.set_x(150).set_y(50).show();

    loop {
        if input.is_pressed(Button::RIGHT) && sprite1_pos.x < agb::display::WIDTH - 16 {
            sprite1_pos.x += 1;
        }
        if input.is_pressed(Button::LEFT) && sprite1_pos.x > 0 {
            sprite1_pos.x -= 1;
        }
        if input.is_pressed(Button::UP) && sprite1_pos.y > 0 {
            sprite1_pos.y -= 1;
        }
        if input.is_pressed(Button::DOWN) && sprite1_pos.y < agb::display::HEIGHT - 16 {
            sprite1_pos.y += 1;
        }

        sprite1
            .set_x(sprite1_pos.x as u16)
            .set_y(sprite1_pos.y as u16);
        agb::display::busy_wait_for_vblank();

        object.commit();
        input.update();
    }
}
