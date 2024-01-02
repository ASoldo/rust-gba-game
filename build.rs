use proc_macro2::TokenStream;
use quote::quote;
use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
use tiled::{Loader, Map};

fn load_tmx(loader: &mut Loader, filename: &str) -> Map {
    println!("cargo:rerun-if-changed={filename}");
    loader.load_tmx_map(filename).expect("failed to load map")
}

fn export_tiles(map: &tiled::Map) -> TokenStream {
    let map_tiles = map.get_layer(0).unwrap().as_tile_layer().unwrap();

    let width = map_tiles.width().unwrap() as usize; // Cast to usize
    let height = map_tiles.height().unwrap() as usize; // Cast to usize

    let map_tiles = (0..(height * width)).map(|pos| {
        let x = pos % width;
        let y = pos / width;

        let tile = map_tiles.get_tile(x as i32, y as i32);

        match tile {
            Some(tile) => {
                let tile_id = tile.id() as i16;
                quote! { #tile_id }
            }
            None => {
                quote! { -1i16 } // Using `u16` explicitly
            }
        }
    });

    // Generate the array of tile IDs in a Rust constant
    quote! {
        pub const TILE_MAP: [i16; #width * #height] = [#(#map_tiles),*];
    }
}

fn main() {
    println!("Hello from build.rs!");
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated.rs");

    let mut tile_loader = Loader::new();
    let map = load_tmx(&mut tile_loader, "assets/FirstMap.tmx");
    let tiles = export_tiles(&map);

    // Generate additional code
    let additional_code = generate_code();

    // Combine all generated code
    let combined_code = quote! {
        #tiles
        #additional_code
    };
    println!("Combined code: {}", combined_code);

    // Create the file and wrap it in a BufWriter
    let file = File::create(&dest_path).expect("Failed to create file");
    let mut writer = BufWriter::new(file);

    // Write the combined generated code to the file
    write!(writer, "{}", combined_code.to_string()).expect("Failed to write to file");
}

/// Generate additional Rust code
fn generate_code() -> TokenStream {
    let message = "Generated Message";
    quote! {
        pub const MESSAGE: &str = #message;
    }
}
