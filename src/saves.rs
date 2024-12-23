use std::env::{self, var};
use std::fs::{self, File};
use std::path::Path;
use std::io::{self, Read, Write};
use std::result::Result;
use std::str::FromStr;

// SHOULD work for any type that has the Display trait, and if not, either change which data is
// stored or implement the trait for said type
macro_rules! generate_save_str {
    ( $( $x:expr ),* ) => {
        {
            let mut save_str = String::new();

            $(
                let cur_str = &(format!("{}", $x) + "\n");
                save_str = save_str + cur_str;
            )*
            save_str
        }
    };
}

// TODO: Parse save string into relative values

use crate::menu::{self, Options};

pub const SAVE_PREFIX: &str = "Rustemon";
pub const SAVE_DIR: &str = "saves";
pub const GLOBAL_OPTIONS_FILE: &str = "global.txt";


pub fn generate_save_path() -> String {
    #[cfg(all(tsrget_os = "windows"))]
    {
        let mut path_string = String::from(env::var("LOCALAPPDATA").unwrap()) + "/" + SAVE_PREFIX;
        path_string = path_string + "/" + SAVE_DIR;

        fs::create_dir_all(&Path::new(&path_string.clone())).expect("Could not create saves directory");
        path_string
    }
    #[cfg(all(target_os = "linux"))]
    {
        let mut path_string = String::from("/usr/share") + "/" + SAVE_PREFIX + "/" + SAVE_DIR;

        fs::create_dir_all(&Path::new(&path_string.clone())).expect("Could not create saves directory");

        path_string

    }
    #[cfg(target_os = "android")]
    {
        //TODO: Use textbox message.
        let prefix = env::var("HOME").expect("Error: Could not read $HOME. Set it in order to enable saves.") +
            "/.local";
        let path_string = prefix + "/" + SAVE_PREFIX + "/" + SAVE_DIR;
        fs::create_dir_all(Path::new(&path_string)).expect("Could not create saves directory");

        path_string
    }

}

// Needs dialog box
pub fn write_save(global_options: &Options) -> io::Result<&str> {
    
    let save_path = generate_save_path();
    let globals_path = save_path + "/" + GLOBAL_OPTIONS_FILE;

    let save_str = generate_save_str!(
            global_options.frame_color_index(),
            global_options.frame_style_index(),
            global_options.text_speed(),
            global_options.master_volume
    );

    match fs::metadata(&globals_path) {
        Ok(_) => {
            // TODO: ASK USER FOR CONFIRMATION USING DIALOG BOX
            
            let mut save_file = File::create(Path::new(&globals_path)).expect("Could not create open file 'saves/global.txt' for writing.");
            
            save_file.set_len(0).expect("Could not clear 'globals.txt'");
            save_file.write_all(save_str.as_bytes()).expect("Could not write to 'globals.txt'");

        },
        Err(_) => {

            let mut save_file = File::create(Path::new(&globals_path)).expect("Could not create save file 'saves/global.txt'.");
            
            save_file.set_len(0).expect("Could not clear 'globals.txt' after creation");
            save_file.write_all(save_str.as_bytes()).expect("Could not write to 'globals.txt' after creation");
        },
    }    

    Ok("Save successful.")
    
} 

pub fn read_save() -> io::Result<Options> { // TODO: Return value should be a tuple containing all
                                            // necessary data to resume game state
    
    let mut default_opts = Options::default();
    let save_path = generate_save_path();
    let globals_path = save_path + "/" + GLOBAL_OPTIONS_FILE;

    #[allow(unused_variables)] //Rust is stupid and can't see that
                               //this is to make the next line easier
                               //to read
    if !Path::new(&globals_path).exists() {
        return Ok(Options::default());
    }

    //let global_opts = File::open(Path::new(&globals_path)).expect("Could not find save file 'saves/global.txt'.");
    let options_string = match fs::read_to_string(&Path::new(&globals_path)) {
        Ok(string) => string,
        Err(_) => return Ok(Options::default()),
    };



    // The values here should be checked so that they are bounded to the expected ranges in order
    // to avoid errors
    for (line_count, line) in options_string.lines().enumerate() {
        match line_count {
            0 => default_opts.set_frame_color(
                match usize::from_str(line) {
                    Ok(num) if num < menu::BORDER_COLORS.len() => num,
                    _ => default_opts.frame_color_index(),
                }
                ),
            1 => default_opts.set_frame_style(
                match usize::from_str(line) {
                    Ok(num) if num < menu::BORDERS.len() => num,
                    _ => default_opts.frame_style_index(),
                }
                ),
            2 => default_opts.set_text_speed(
                match u8::from_str(line) {
                    Ok(num) => {
                        if num <= 5 {
                            num
                        } else {
                            5
                        }
                    },
                    _ => default_opts.text_speed(),
                }
                ),
            3 => default_opts.master_volume = 
                match f32::from_str(line) {
                    Ok(num) => {
                        if num > 1.0 {
                            1.0
                        } else if num < 0.0 {
                            0.0
                        } else {
                            num
                        }
                    },
                    _ => default_opts.master_volume,
                },
            _ => {/* TODO: Make a dialog box that says, "Too many options in options file. Exceeding lines ignored."*/},            
        }

    }

    Ok(default_opts)
}

