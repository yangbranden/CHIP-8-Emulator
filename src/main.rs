extern crate minifb;
extern crate rand;
mod interface;
use interface::Interface;
mod chip8;
use chip8::Chip8;
use minifb::{Key, Scale};
use std::path::Path;

fn main() {
    // Get rom_filepath from command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let rom_filepath = if args.len() > 1 {
        let filepath = &args[1];
        println!("Found program: {}", filepath);
        filepath
    } else {
        println!(
            "\nNo arguments provided; please provide a ROM file path using the following syntax:\n\tcargo run -- <path_to_rom>\n"
        );
        std::process::exit(1);
    };


    // ======================= USER SETTINGS =======================
    let scale_factor = Scale::X16; // Scaling size for screen (original is 64x32; factor of 16 will make it 1024x512)
    let cpu_hz = 200.0; // Instructions per second
    let display_hz = 60.0; // Frames per second
    let muted = false; // Whether or not to mute sound
    let debug_mode = true; // Enable debug mode to print additional information
    // =============================================================
    

    // Create an Interface instance with specified scaling
    let interface = Interface::new(scale_factor);
    
    // Create a Chip8 instance with our Interface instance
    let mut chip8 = Chip8::new(interface);
    
    // Define the path to the ROM file and load it into memory
    let rom_path = Path::new(&rom_filepath);
    chip8.load_program(rom_path);
    
    // Calculate the number of CPU cycles per frame
    let cycles_per_frame = (cpu_hz / display_hz) as usize;
    
    // Additional settings
    chip8.interface.muted = muted;
    chip8.interface.debug_mode = debug_mode;
    
    // Main loop; exit if window is closed or Escape is pressed
    while chip8.interface.window.is_open() && !chip8.interface.window.is_key_down(Key::Escape) {
        // Process user input
        chip8.interface.process_keys();

        // Process CPU cycles
        for _ in 0..cycles_per_frame {
            chip8.emulate_cycle();
        }

        // Render the display
        chip8.interface.render_screen();
    }
}
