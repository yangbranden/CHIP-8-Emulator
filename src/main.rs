extern crate minifb;
extern crate rand;
mod interface;
use interface::Interface;
mod chip8;
use chip8::Chip8;
use minifb::{Key, Scale};
use std::path::Path;
use std::time::{Duration, Instant};

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

    // Define scaling size for screen (original is 64x32, so scale_factor = 10 will make it 640x320)
    let scale_factor = Scale::X16;

    // Create an Interface instance with specified scaling
    let interface = Interface::new(scale_factor);

    // Create a Chip8 instance with our Graphics instance
    let mut chip8 = Chip8::new(interface);

    // Define the path to the ROM file and load it into memory
    let rom_path = Path::new(&rom_filepath);
    chip8.load_program(rom_path);

    // Define clock speeds
    let cpu_hz = 500.0; // Instructions per second
    let display_hz = 60.0; // Frames per second

    let cpu_interval = Duration::from_secs_f64(1.0 / cpu_hz);
    let display_interval = Duration::from_secs_f64(1.0 / display_hz);

    let mut last_cpu_tick = Instant::now();
    let mut last_display_tick = Instant::now();

    // Main loop; exit if window is closed or Escape is pressed
    while chip8.interface.window.is_open() && !chip8.interface.window.is_key_down(Key::Escape) {
        let current_time = Instant::now();

        // Process CPU cycles
        if current_time - last_cpu_tick >= cpu_interval {
            let keys = chip8.interface.window.get_keys();
            chip8.interface.process_keys(keys);

            // Update timers and execute instruction
            chip8.emulate_cycle();

            last_cpu_tick = current_time;
        }

        // Update display
        if current_time - last_display_tick >= display_interval {
            chip8.interface.render_screen();
            last_display_tick = current_time;
        }
    }
}
