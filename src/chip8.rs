use crate::interface::Interface;
// use rand::Rng;
use std::{fs::read, path::Path};

// Memory layout according to https://austinmorlan.com/posts/chip8_emulator/#4k-bytes-of-memory
// 0x000-0x1FF: CHIP-8 interpreter
//              (in our modern emulator we will just never write to or read to this area)
// 0x050-0x0A0: Storage space for the 16 built-in characters (0 through F),
//              which we will need to manually put into our memory because ROMs will be looking for those characters.
// 0x200-0xFFF: Instructions from the ROM will be stored starting at 0x200,
//              and anything left after the ROM’s space is free to use.
pub struct Chip8 {
    memory: [u8; 4096],       // 4KB memory
    v: [u8; 16],              // 16 general-purpose 8-bit registers (V0 through VF)
    i: u16,                   // Index register
    pc: u16,                  // Program counter
    stack: [u16; 16],         // Call stack
    sp: u8,                   // Stack pointer
    delay_timer: u8,          // Delay timer
    sound_timer: u8,          // Sound timer
    pub interface: Interface, // See interface.rs for rendering display and receiving input
}

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl Chip8 {
    pub fn new(interface: Interface) -> Self {
        let mut chip8 = Chip8 {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200, // Programs typically start at memory address 0x200
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            interface: interface,
        };

        // Load fontset into memory
        chip8.memory[0..80].copy_from_slice(&FONTSET);

        chip8
    }

    pub fn load_program(&mut self, program: &Path) {
        // We load program into memory starting at 0x200
        let start = 0x200;

        // Read binary data from Path object
        match read(program) {
            Ok(contents) => {
                // `contents` is a Vec<u8> containing the binary data of the file
                // You can now process the binary data as needed
                println!("File read successfully. Size: {} bytes", contents.len());
                self.memory[start..(start + contents.len())].copy_from_slice(&contents);
            }
            Err(e) => {
                eprintln!("Error reading file: {}", e);
            }
        }
    }

    pub fn emulate_cycle(&mut self) {
        // 1. Fetch instruction
        let instruction = self.fetch_instruction();

        // 2. Execute instruction
        self.execute_instruction(instruction);

        // 3. Update timers
        self.update_timers();
    }

    fn fetch_instruction(&mut self) -> u16 {
        // An instruction is two bytes but memory is addressed as a single byte,
        // so when we fetch an instruction from memory we need to fetch a byte from PC
        // and a byte from PC+1 and connect them into a single value;
        let high_byte = self.memory[self.pc as usize] as u16;
        let low_byte = self.memory[(self.pc + 1) as usize] as u16;

        // Read instruction from top 4 bits
        let instruction = (high_byte << 8) | low_byte;

        instruction
    }

    fn execute_instruction(&mut self, opcode: u16) {
        // For the sake of (at least my) understanding, opcode == instruction
        // (it's not really but there isn't a specific "opcode" section of the instruction, it is kinda dependent on a lot of things)

        // Increment PC to point to the next instruction before we execute anything
        self.pc += 2;

        // http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#3.1
        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x00FF {
                    0x00E0 => self.cls(), // CLS: Clear the display
                    0x00EE => self.ret(), // RET: Return from subroutine
                    _ => println!("Unknown opcode: {:X}", opcode),
                }
            }
            0x1000 => self.jp(opcode),   // JP: Jump to address NNN
            0x2000 => self.call(opcode), // CALL: Call subroutine at address NNN
            0x3000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize; // x
                let kk = (opcode & 0x00FF) as u8; // kk
                self.se_vx(x, kk); // SE Vx, byte: Skip next instruction if Vx == kk
            }
            0x4000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize; // x
                let kk = (opcode & 0x00FF) as u8; // kk
                self.sne_vx(x, kk); // SNE Vx, byte: Skip next instruction if Vx != kk
            }
            0x5000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize; // x
                let y = ((opcode & 0x00F0) >> 4) as usize; // y
                self.se_vx_vy(x, y); // SE Vx, Vy: Skip next instruction if Vx == Vy
            }
            0x6000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize; // x
                let kk = (opcode & 0x00FF) as u8; // Extract byte (kk)
                self.ld_vx(x, kk); // LD Vx, byte: Set Vx = kk
            }
            0x7000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize; // x
                let kk = (opcode & 0x00FF) as u8; // Extract byte (kk)
                self.add_vx(x, kk); // ADD Vx, byte: Set Vx = Vx + kk
            }
            0x8000 => {
                match opcode & 0x000F {
                    0x0000 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        let y = ((opcode & 0x00F0) >> 4) as usize; // y
                        self.ld_vx_vy(x, y); // LD Vx, Vy: Set Vx = Vy
                    }
                    0x0001 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        let y = ((opcode & 0x00F0) >> 4) as usize; // y
                        self.or_vx_vy(x, y); // OR Vx, Vy: Set Vx = Vx OR Vy
                    }
                    0x0002 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        let y = ((opcode & 0x00F0) >> 4) as usize; // y
                        self.and_vx_vy(x, y); // AND Vx, Vy: Set Vx = Vx AND Vy
                    }
                    0x0003 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        let y = ((opcode & 0x00F0) >> 4) as usize; // y
                        self.xor_vx_vy(x, y); // XOR Vx, Vy: Set Vx = Vx XOR Vy
                    }
                    0x0004 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        let y = ((opcode & 0x00F0) >> 4) as usize; // y
                        self.add_vx_vy(x, y); // ADD Vx, Vy: Set Vx = Vx + Vy
                    }
                    0x0005 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        let y = ((opcode & 0x00F0) >> 4) as usize; // y
                        self.sub_vx_vy(x, y); // SUB Vx, Vy: Set Vx = Vx - Vy
                    }
                    0x0006 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.shr_vx(x); // SHR Vx: Set Vx = Vx SHR 1
                    }
                    0x0007 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        let y = ((opcode & 0x00F0) >> 4) as usize; // y
                        self.subn_vx_vy(x, y); // SUBN Vx, Vy: Set Vx = Vy - Vx
                    }
                    0x000E => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.shl_vx(x); // SHL Vx: Set Vx = Vx SHL 1
                    }
                    _ => println!("Unknown opcode: {:X}", opcode),
                }
            }
            0x9000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize; // x
                let y = ((opcode & 0x00F0) >> 4) as usize; // y
                self.sne_vx_vy(x, y); // SNE Vx, Vy: Skip next instruction if Vx != Vy
            }
            0xA000 => {
                let nnn = opcode & 0x0FFF;
                self.ld_i(nnn); // LD I, addr: Set I = nnn
            }
            0xB000 => {
                let nnn = opcode & 0x0FFF;
                self.jp_v0(nnn); // JP V0, addr: Jump to location nnn + V0
            }
            0xC000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize; // x
                let kk = (opcode & 0x00FF) as u8; // kk
                self.rnd(x, kk); // RND Vx, byte: Set Vx = random byte AND kk
            }
            0xD000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize; // x
                let y = ((opcode & 0x00F0) >> 4) as usize; // y
                let n = (opcode & 0x000F) as u8; // n
                self.drw(x, y, n); // DRW Vx, Vy, nibble: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            }
            0xE000 => {
                match opcode & 0x00FF {
                    0x009E => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.skp(x); // SKP Vx: Skip next instruction if key with the value of Vx is pressed
                    }
                    0x00A1 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.sknp(x); // SKNP Vx: Skip next instruction if key with the value of Vx is not pressed
                    }
                    _ => println!("Unknown opcode: {:X}", opcode),
                }
            }
            0xF000 => {
                match opcode & 0x00FF {
                    0x0007 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.ld_vx_dt(x); // LD Vx, DT: Set Vx = delay timer value
                    }
                    0x000A => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.ld_vx_k(x); // LD Vx, K: Wait for a key press, store the value of the key in Vx
                    }
                    0x0015 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.ld_dt_vx(x); // LD DT, Vx: Set delay timer = Vx
                    }
                    0x0018 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.ld_st_vx(x); // LD ST, Vx: Set sound timer = Vx
                    }
                    0x001E => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.add_i_vx(x); // ADD I, Vx: Set I = I + Vx
                    }
                    0x0029 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.ld_f_vx(x); // LD F, Vx: Set I = location of sprite for digit Vx
                    }
                    0x0033 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.ld_b_vx(x); // LD B, Vx: Store BCD representation of Vx in memory locations I, I+1, and I+2
                    }
                    0x0055 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.ld_i_vx(x); // LD [I], Vx: Store registers V0 through Vx in memory starting at location I
                    }
                    0x0065 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize; // x
                        self.ld_vx_i(x); // LD Vx, [I]: Read registers V0 through Vx from memory starting at location I
                    }
                    _ => println!("Unknown opcode: {:X}", opcode),
                }
            }
            _ => println!("Unknown opcode: {:X}", opcode),
        }
    }

    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.interface.set_beep(true);
            self.sound_timer -= 1;
        } else {
            self.interface.set_beep(false);
        }
    }

    fn cls(&mut self) {
        // Clear the screen buffer
        self.interface.screen.fill(0);
    }

    fn ret(&mut self) {
        // Return from subroutine
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn jp(&mut self, opcode: u16) {
        // Jump to address NNN
        let nnn = opcode & 0x0FFF;
        self.pc = nnn;
    }

    fn call(&mut self, opcode: u16) {
        // Call subroutine at address NNN
        let nnn = opcode & 0x0FFF;
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = nnn;
    }

    fn se_vx(&mut self, x: usize, kk: u8) {
        // Skip next instruction if Vx == kk
        if self.v[x] == kk {
            self.pc += 2;
        }
    }

    fn sne_vx(&mut self, x: usize, kk: u8) {
        // Skip next instruction if Vx != kk
        if self.v[x] != kk {
            self.pc += 2;
        }
    }

    fn se_vx_vy(&mut self, x: usize, y: usize) {
        // Skip next instruction if Vx == Vy
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    fn ld_vx(&mut self, x: usize, kk: u8) {
        // Set Vx = kk
        self.v[x] = kk;
    }

    fn add_vx(&mut self, x: usize, kk: u8) {
        // Set Vx = Vx + kk
        self.v[x] = self.v[x].wrapping_add(kk);
    }

    fn ld_vx_vy(&mut self, x: usize, y: usize) {
        // Set Vx = Vy
        self.v[x] = self.v[y];
    }

    fn or_vx_vy(&mut self, x: usize, y: usize) {
        // Set Vx = Vx OR Vy
        self.v[x] |= self.v[y];
    }

    fn and_vx_vy(&mut self, x: usize, y: usize) {
        // Set Vx = Vx AND Vy
        self.v[x] &= self.v[y];
    }

    fn xor_vx_vy(&mut self, x: usize, y: usize) {
        // Set Vx = Vx XOR Vy
        self.v[x] ^= self.v[y];
    }

    fn add_vx_vy(&mut self, x: usize, y: usize) {
        // Set Vx = Vx + Vy, set VF = carry
        let (result, overflow) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = result;
        self.v[0xF] = overflow as u8;
    }

    fn sub_vx_vy(&mut self, x: usize, y: usize) {
        // Set Vx = Vx - Vy, set VF = NOT borrow
        let (result, overflow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = result;
        self.v[0xF] = !overflow as u8;
    }

    fn shr_vx(&mut self, x: usize) {
        // Set Vx = Vx SHR 1
        self.v[0xF] = self.v[x] & 0x1;
        self.v[x] >>= 1;
    }

    fn subn_vx_vy(&mut self, x: usize, y: usize) {
        // Set Vx = Vy - Vx, set VF = NOT borrow
        let (result, overflow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = result;
        self.v[0xF] = !overflow as u8;
    }

    fn shl_vx(&mut self, x: usize) {
        // Set Vx = Vx SHL 1
        self.v[0xF] = (self.v[x] & 0x80) >> 7;
        self.v[x] <<= 1;
    }

    fn sne_vx_vy(&mut self, x: usize, y: usize) {
        // Skip next instruction if Vx != Vy
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    fn ld_i(&mut self, nnn: u16) {
        // Set I = nnn
        self.i = nnn;
    }

    fn jp_v0(&mut self, nnn: u16) {
        // Jump to location nnn + V0
        self.pc = nnn + self.v[0] as u16;
    }

    fn rnd(&mut self, x: usize, kk: u8) {
        // Set Vx = random byte AND kk
        let random_byte: u8 = rand::random();
        self.v[x] = random_byte & kk;
    }

    fn drw(&mut self, x: usize, y: usize, n: u8) {
        println!("Drawing {}-byte sprite at ({}, {})", n, self.v[x], self.v[y]);

        // Get the starting coordinates from the input registers
        let start_x = self.v[x] as usize;
        let start_y = self.v[y] as usize;

        // Reset the collision flag (VF)
        self.v[0xF] = 0;

        // Loop over each of the n rows of the sprite (one row per byte)
        for yline in 0..n {
            // Get the 8-bit sprite data for the current row from memory
            let sprite_data = self.memory[(self.i + yline as u16) as usize];

            // Loop over the 8 bits (pixels) of the current sprite row
            for xline in 0..8 {
                // Check if the current bit/pixel is set to 1
                if (sprite_data & (0x80 >> xline)) != 0 {
                    // Calculate the final screen coordinates for the pixel, applying wrapping
                    let final_x = (start_x + xline) % 64;
                    let final_y = (start_y + yline as usize) % 32;

                    // Convert 2D coordinates to a 1D index for the screen buffer
                    let index = final_y * 64 + final_x;

                    // Check for collision: if the pixel on screen is already on, set the VF flag
                    if self.interface.screen[index] == 0xFFFFFF {
                        self.v[0xF] = 1;
                    }

                    // XOR the pixel onto the screen buffer
                    self.interface.screen[index] ^= 0xFFFFFF;
                }
            }
        }
    }

    fn skp(&mut self, x: usize) {
        // Skip next instruction if key with the value of Vx is pressed
        if self.interface.keypad[self.v[x] as usize] {
            self.pc += 2;
        }
    }

    fn sknp(&mut self, x: usize) {
        // Skip next instruction if key with the value of Vx is not pressed
        if !self.interface.keypad[self.v[x] as usize] {
            self.pc += 2;
        }
    }

    fn ld_vx_dt(&mut self, x: usize) {
        // Set Vx = delay timer value
        self.v[x] = self.delay_timer;
    }

    fn ld_vx_k(&mut self, x: usize) {
        // Wait for a key press, store the value of the key in Vx
        for i in 0..16 {
            if self.interface.keypad[i] {
                self.v[x] = i as u8;
                return;
            }
        }
        self.pc -= 2;
    }

    fn ld_dt_vx(&mut self, x: usize) {
        // Set delay timer = Vx
        self.delay_timer = self.v[x];
    }

    fn ld_st_vx(&mut self, x: usize) {
        // Set sound timer = Vx
        self.sound_timer = self.v[x];
    }

    fn add_i_vx(&mut self, x: usize) {
        // Set I = I + Vx
        self.i += self.v[x] as u16;
    }

    fn ld_f_vx(&mut self, x: usize) {
        // Set I = location of sprite for digit Vx
        self.i = self.v[x] as u16 * 5;
    }

    fn ld_b_vx(&mut self, x: usize) {
        // Store BCD representation of Vx in memory locations I, I+1, and I+2
        let vx = self.v[x];
        self.memory[self.i as usize] = vx / 100;
        self.memory[(self.i + 1) as usize] = (vx / 10) % 10;
        self.memory[(self.i + 2) as usize] = vx % 10;
    }

    fn ld_i_vx(&mut self, x: usize) {
        // Store registers V0 through Vx in memory starting at location I
        for i in 0..=x {
            self.memory[(self.i + i as u16) as usize] = self.v[i];
        }
    }

    fn ld_vx_i(&mut self, x: usize) {
        // Read registers V0 through Vx from memory starting at location I
        for i in 0..=x {
            self.v[i] = self.memory[(self.i + i as u16) as usize];
        }
    }
}
