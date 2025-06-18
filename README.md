# CHIP-8

CHIP-8 Emulator made using Rust

used minifb for graphics/user input, rodio for audio

## About CHIP-8

CHIP-8 (see [Wikipedia page here](https://en.wikipedia.org/wiki/CHIP-8)) is an interpreted programming language developed by Joseph Weisbecker in the 1970s. It was initially used on the COSMAC VIP and Telmac 1800 8-bit microcomputers to make game programming easier.

![wikipediaimage](https://upload.wikimedia.org/wikipedia/commons/thumb/5/54/Space_intercept.png/1280px-Space_intercept.png)

So actually, a CHIP-8 "emulator" is *technically* an interpreter for the language. However, most people refer to building a CHIP-8 interpreter as building an "emulator" or "virtual machine", because the process of building one is very similar to the process of building a simple emulator.

CHIP-8 emulators are commonly referred to as the "Hello World" of emulator development. 

Some key features about CHIP-8 include:
- 16 8-bit wide registers (along with a 2-byte "I" register used to store addresses and a 2-byte PC register for storing the current instruction)
- 4096 bytes of memory (where programs and graphics data reside)
- A simple stack of 16 16-bit values used to store subroutines (function calls) and their return addresses
- A black-and-white graphics system with a display resolution of 64x32 pixels
- A keypad with 16 keys for user input

## Installation/Compilation

To try out and compile this emulator, make sure you have cargo installed:

```
curl https://sh.rustup.rs -sSf | sh
```

Download this repository using:

```
git clone https://github.com/yangbranden/CHIP-8.git
```

Then download some games (`.ch8` file format) and run using the following syntax:

```
cargo run -- <path_to_rom>
```

## Games
Repositories with CHIP-8 games to download/try

https://github.com/dmatlack/chip8/tree/master/roms/games

https://github.com/kripod/chip8-roms/tree/master/games


![rustlangmeme](https://i.imgur.com/acnHrCO.png)
