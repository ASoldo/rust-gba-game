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
    display::{
        object::{Graphics, Tag, TagMap},
        tiled::TileFormat,
    },
    display::{
        tiled::{InfiniteScrolledMap, RegularBackgroundSize, TiledMap},
        window::WinIn,
        Priority, HEIGHT, WIDTH,
    },
    fixnum::num,
    fixnum::{Rect, Vector2D},
    include_aseprite, include_background_gfx, include_wav,
    input::Button,
    interrupt::VBlank,
    println,
    sound::mixer::{Frequency, SoundChannel},
    Gba,
};
use agb_tracker::{include_xm, Track, Tracker};
extern crate alloc;
use alloc::boxed::Box;

const SWORD_PICKUP: &[u8] = include_wav!("sfx/slime_death.wav");
const THEME: Track = include_xm!("sfx/gwilym-theme2.xm");

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

include_background_gfx!(tileset1, tiles => deduplicate "assets/Tileset1.aseprite");

const GRAPHICS: &Graphics = include_aseprite!("assets/Sprites.aseprite", "assets/anim.aseprite");
const TAG_MAP: &TagMap = GRAPHICS.tags();
const SPRITE1: &Tag = &GRAPHICS.tags().get("Sprite1");
const SPRITE2: &Tag = &GRAPHICS.tags().get("Sprite2");

const ANIM: &Tag = TAG_MAP.get("Sprite1");

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
// #[cfg_attr(feature = "entry", agb::entry)]
#[agb::entry]
fn entry(gba: Gba) -> ! {
    main(gba);
}

fn main(mut gba: Gba) -> ! {
    // Initialize the mixer
    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();

    // Initialize the sound channel
    let mut channel = SoundChannel::new(SWORD_PICKUP);
    channel.playback(num!(1.0));

    // Play the sound once
    mixer.play_sound(channel);

    let mut tracker = Tracker::new(&THEME);

    let mut start: bool = false;
    println!("{}", MESSAGE);
    let vblank = VBlank::get();
    let tileset = tileset1::tiles.tiles;
    let (gfx, mut vram) = gba.display.video.tiled0();
    vram.set_background_palettes(tileset1::PALETTES);

    // Initialize InfiniteScrolledMap for the background
    let mut infinite_bg = InfiniteScrolledMap::new(
        gfx.background(
            Priority::P3,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        ),
        Box::new(|pos| {
            let index = pos.y * 30 + pos.x;
            let tile_id = if pos.x < 30 && pos.y < 20 && pos.x >= 0 && pos.y >= 0 {
                // Within bounds, fetch the corresponding tile ID
                *TILE_MAP.get(index as usize).unwrap_or(&1)
            } else {
                // Out of bounds, use tile ID 1 (or another ID representing an edge or empty tile)
                7
            };
            (&tileset, tileset1::tiles.tile_settings[tile_id as usize])
        }),
    );

    // Set the initial position of the scrolled background
    let start_pos = (0, 0).into();
    infinite_bg.init(&mut vram, start_pos, &mut || {});

    // Show the infinite background
    infinite_bg.commit(&mut vram);
    infinite_bg.show();

    // let mut bg = gfx.background(
    //     Priority::P3,
    //     RegularBackgroundSize::Background32x32,
    //     tileset.format(),
    // );

    let mut bg2 = gfx.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        tileset.format(),
    );

    let mut bg3 = gfx.background(
        Priority::P1,
        RegularBackgroundSize::Background32x32,
        tileset.format(),
    );

    // for y in 0..20u16 {
    //     for x in 0..30u16 {
    //         // Calculate the index in the 1D TILE_MAP array based on x, y coordinates
    //         let index = y as usize * 30 + x as usize; // Assuming row-major order
    //
    //         // Get the tile ID from the TILE_MAP using the calculated index
    //         let tile_id = TILE_MAP[index] as usize; // Cast to usize to use as an index
    //
    //         // Set the tile on the background using the tile ID from TILE_MAP
    //         // Make sure that tileset1::tiles.tile_settings[tile_id] gives you the correct setting for that tile ID
    //         bg.set_tile(
    //             &mut vram,
    //             (x, y).into(),
    //             &tileset,
    //             tileset1::tiles.tile_settings[tile_id],
    //         );
    //     }
    // }

    for i in 0..3u16 {
        bg3.set_tile(
            &mut vram,
            (i as u16, 0 as u16).into(),
            &tileset,
            tileset1::tiles.tile_settings[36],
        );
    }

    bg2.set_tile(
        &mut vram,
        (12 as u16, 12 as u16).into(),
        &tileset,
        tileset1::tiles.tile_settings[37],
    );

    let mut window = gba.display.window.get();
    window
        .win_in(WinIn::Win0)
        .set_background_enable(bg3.background(), true)
        // .set_background_enable(bg.background(), true)
        .set_background_enable(infinite_bg.background(), true)
        .set_object_enable(true)
        .set_blend_enable(true)
        .set_position(&Rect::new((0, 0).into(), (24, 8).into()))
        // .enable()
    ;

    window
        .win_out()
        // .set_background_enable(bg.background(), true)
        .set_background_enable(infinite_bg.background(), true)
        .set_background_enable(bg2.background(), true)
        .set_object_enable(true)
        .set_blend_enable(true)
        .enable();

    // let mut blend = gba.display.blend.get();
    // blend
    //     .set_backdrop_enable(Layer::Top, true)
    //     .set_background_enable(Layer::Bottom, bg.background(), true)
    //     .set_blend_mode(BlendMode::Normal);

    // Demo Game
    println!("Demo Gameee");
    let soldo: i32 = 10;
    println!("{:?}", soldo);
    let so: DemoLog = DemoLog { demo_id: 1 };
    println!("{:?}", so.demo_id);

    // agb::no_game(gba);

    let mut input = agb::input::ButtonController::new();
    let object = gba.display.object.get_managed();
    let mut sprite1 = object.object_sprite(SPRITE1.sprite(0));
    let mut sprite1_pos = Position {
        x: WIDTH / 2 - 8,
        y: HEIGHT / 2 - 8,
    };
    sprite1.set_priority(Priority::P3);
    sprite1
        .set_x(sprite1_pos.x as u16)
        .set_y(sprite1_pos.y as u16)
        .show();

    let mut sprite2 = object.object_sprite(SPRITE2.sprite(0));
    sprite2.set_priority(Priority::P3);
    sprite2.set_x(150).set_y(50).show();

    // bg.commit(&mut vram);
    // bg.show();
    bg2.commit(&mut vram);
    bg2.show();
    // Find the frame count for Sprite1
    let sprite1_frame_count = ANIMATION_FRAMES
        .iter()
        .find(|&&(name, _)| name == "Sprite1")
        .map(|&(_, count)| count as usize)
        .unwrap_or(1); // Default to 1 if not found

    let mut sprite1_anim_frame = 0;

    let mut bg_position = Vector2D::new(0, 0);

    // Define the centering bounds as a rectangle
    let centering_bounds = Rect::new(
        Vector2D::new(WIDTH as i32 / 4, HEIGHT as i32 / 4),
        Vector2D::new(WIDTH as i32 / 2, HEIGHT as i32 / 2),
    );

    loop {
        tracker.step(&mut mixer);
        mixer.frame();

        sprite1_anim_frame = (sprite1_anim_frame + 1) % (16 * sprite1_frame_count);

        let frame_index = ANIM.sprite(sprite1_anim_frame / 16);
        let sprite_frame = object.sprite(frame_index);
        sprite1.set_sprite(sprite_frame);

        if input.is_pressed(Button::RIGHT) && sprite1_pos.x < WIDTH - 16 {
            // bg_position.x += 1;
            sprite1_pos.x += 1;
            sprite1.set_hflip(false);
        }
        if input.is_pressed(Button::LEFT) && sprite1_pos.x > 0 {
            // bg_position.x -= 1;
            sprite1_pos.x -= 1;
            sprite1.set_hflip(true);
        }
        if input.is_pressed(Button::UP) && sprite1_pos.y > 0 {
            // bg_position.y -= 1;
            sprite1_pos.y -= 1;
        }
        if input.is_pressed(Button::DOWN) && sprite1_pos.y < HEIGHT - 16 {
            // bg_position.y += 1;
            sprite1_pos.y += 1;
        }
        if input.is_just_pressed(Button::START) {
            start = !start;
            if start {
                bg3.commit(&mut vram);
                bg3.show();
                window.win_in(WinIn::Win0).enable();
            } else {
                bg3.hide();
                window.win_in(WinIn::Win0).disable();
            }
            window.commit();
        }
        if input.is_just_pressed(Button::A) {
            let mut channel = SoundChannel::new(SWORD_PICKUP);
            channel.playback(num!(1.0));
            mixer.play_sound(channel);
        }

        sprite1
            .set_x(sprite1_pos.x as u16)
            .set_y(sprite1_pos.y as u16);

        // bg_position.x += 1;
        // bg_position.y -= 1;

        // let center = Vector2D::new(WIDTH as i32 / 2, HEIGHT as i32 / 2);
        // if sprite1_pos.x < center.x - centering_bounds.size.x / 2 {
        //     bg_position.x = sprite1_pos.x + centering_bounds.size.x / 2 - center.x;
        // } else if sprite1_pos.x > center.x + centering_bounds.size.x / 2 {
        //     bg_position.x = sprite1_pos.x - center.x - centering_bounds.size.x / 2;
        // }
        //
        // if sprite1_pos.y < center.y - centering_bounds.size.y / 2 {
        //     bg_position.y = sprite1_pos.y + centering_bounds.size.y / 2 - center.y;
        // } else if sprite1_pos.y > center.y + centering_bounds.size.y / 2 {
        //     bg_position.y = sprite1_pos.y - center.y - centering_bounds.size.y / 2;
        // }
        infinite_bg.set_pos(&mut vram, bg_position);
        vblank.wait_for_vblank();

        infinite_bg.commit(&mut vram);
        // blend.commit();
        object.commit();
        input.update();
    }
}
