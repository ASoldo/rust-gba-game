use proc_macro2::TokenStream;
use quote::quote;
use serde_json::Value;
use std::{
    env, fs,
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
                let tile_id = tile.id() as u16;
                quote! { #tile_id }
            }
            None => {
                quote! { 0u16 } // Using `u16` explicitly
            }
        }
    });

    // Generate the array of tile IDs in a Rust constant
    quote! {
        pub const TILE_MAP: [u16; #width * #height] = [#(#map_tiles),*];
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
    let frame_data = parse_aseprite_json(&["assets/anim.json", "assets/Sprites.json"]);

    // Combine all generated code
    let combined_code = quote! {
        #tiles
        #additional_code
        #frame_data
    };
    println!("Combined code: {}", combined_code);

    // Create the file and wrap it in a BufWriter
    let file = File::create(&dest_path).expect("Failed to create file");
    let mut writer = BufWriter::new(file);

    // Write the combined generated code to the file
    write!(writer, "{}", combined_code.to_string()).expect("Failed to write to file");
}

fn parse_aseprite_json(paths: &[&str]) -> TokenStream {
    let mut all_frame_data_tokens = Vec::new();

    for path in paths {
        let aseprite_file_path = Path::new(path);
        // Inform Cargo to rerun this build script if the Aseprite JSON file changes
        println!("cargo:rerun-if-changed={path}");

        // Read the Aseprite JSON file
        let file_content =
            fs::read_to_string(aseprite_file_path).expect("Failed to read Aseprite file");

        // Parse the file content as JSON
        let json: Value = serde_json::from_str(&file_content).expect("Failed to parse JSON");

        // Collect frame data for each tag
        if let Some(tags) = json["meta"]["frameTags"].as_array() {
            for tag in tags {
                let tag_name = tag["name"].as_str().unwrap_or("unknown").to_string();
                let frame_count =
                    tag["to"].as_i64().unwrap_or(0) - tag["from"].as_i64().unwrap_or(0) + 1;
                all_frame_data_tokens.push(quote! { (#tag_name, #frame_count) });
            }
        }
    }

    // Convert the collected tokens into a TokenStream
    let frame_data = quote! { [#(#all_frame_data_tokens),*] };

    // Generate the array of frame data in a Rust constant
    quote! {
        pub const ANIMATION_FRAMES: [(&str, i64); #frame_data.len()] = #frame_data;
    }
}

/// Generate additional Rust code
fn generate_code() -> TokenStream {
    let message = "Generated Message";
    quote! {
        pub const MESSAGE: &str = #message;
    }
}
