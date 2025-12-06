# Open Source Keyboard Appliance powered by Rust

OSKAR is a firmware for the Raspberry Pi Pico that combines the 9elements picoprog project and additional HID code into a firmware for our multi-feature Macro-Keypad.
## Prerequisites

Before you can compile and use OSKAR, you need to install the following dependencies:

- Rust and Cargo: Follow the instructions on the [official Rust website](https://www.rust-lang.org/tools/install) to install Rust and Cargo.
- Install flip-link and elf2uf2

```sh
# Only Linux
sudo apt install build-essential libudev-dev pkg-config

# All systems
cargo install flip-link elf2uf2-rs
```

## Compiling the Firmware

To compile the firmware, follow these steps:

1. Clone the repository:

```sh
git clone https://github.com/9elements/OSKAR.git
cd OSKAR
```

2. Build the firmware:

```sh
cargo run --release
```

3. The compiled binary will be located in the `target/thumbv6m-none-eabi/release` directory.

## Flashing the Firmware

To flash the firmware onto the Raspberry Pi Pico, follow these steps:

1. Connect the Raspberry Pi Pico to your computer while holding the BOOTSEL button. This will put the Pico into USB mass storage mode.

2. Copy the UF2 file to the Pico:

```sh
# Linux
cp target/thumbv6m-none-eabi/release/oskar.uf2 /path/to/pi/volume

# macOS
cp target/thumbv6m-none-eabi/release/oskar.uf2 /Volumes/RPI-RP2/
```

3. The Pico will automatically reboot and start running the new firmware.

## Usage

### Modes

The Device features a 3-position selection switch on its left side. When the switch is moved all the way to the back it simply acts as the picoprog featuring SPI and UART capabilities (Green LED). Moved all the way to the front it acts as a MIDI Controller (Red LED). With the switch in the middle position the device combines both features at once (purple LED).

### MIDI Controller

The firmware configures the device as a USB MIDI controller that sends MIDI Control Change (CC) messages. The default configuration uses MIDI channel 15 to avoid conflicts with other MIDI devices.

#### Default Configuration

At the top of the file `src/hid.rs` there are two constant structs:

**MIDI Channel Configuration:**
```rust
const MIDI_CONFIG: MidiConfig = MidiConfig {
    channel: 14, // MIDI channel 15 (0-indexed)
};
```

**Key Layout Configuration:**
```rust
const KEYLAYOUT: KeyLayout = KeyLayout {
    encoder_left: MidiCC { controller: 1, value_on: 127, value_off: 0 },    // CC 1 (Modulation)
    encoder_right: MidiCC { controller: 2, value_on: 127, value_off: 0 },   // CC 2 (Breath Controller)
    encoder_button: MidiCC { controller: 3, value_on: 127, value_off: 0 },  // CC 3
    key1: MidiCC { controller: 20, value_on: 127, value_off: 0 },           // CC 20
    key2: MidiCC { controller: 21, value_on: 127, value_off: 0 },           // CC 21
    key3: MidiCC { controller: 22, value_on: 127, value_off: 0 },           // CC 22
};
```

#### Customization

You can customize the MIDI configuration by editing these constants in `src/hid.rs`:

1. **Change MIDI Channel**: Modify the `channel` value in `MIDI_CONFIG`. Note that channels are 0-indexed, so channel 15 is represented as 14.

2. **Change CC Numbers**: Modify the `controller` value for each key. Valid values are 0-127.

3. **Change CC Values**: Modify `value_on` (sent when button is pressed or encoder rotated) and `value_off` (sent when button is released or after encoder rotation).

Example - Using different CC numbers:
```rust
const KEYLAYOUT: KeyLayout = KeyLayout {
    encoder_left: MidiCC { controller: 10, value_on: 127, value_off: 0 },
    encoder_right: MidiCC { controller: 11, value_on: 127, value_off: 0 },
    encoder_button: MidiCC { controller: 12, value_on: 127, value_off: 0 },
    key1: MidiCC { controller: 70, value_on: 127, value_off: 0 },
    key2: MidiCC { controller: 71, value_on: 127, value_off: 0 },
    key3: MidiCC { controller: 72, value_on: 127, value_off: 0 },
};
```

#### Behavior

- **Encoder rotation**: Sends a CC message with `value_on` followed immediately by a CC message with `value_off`, creating a momentary trigger effect.
- **Button press**: Sends a CC message with `value_on`.
- **Button release**: Sends a CC message with `value_off`.

This allows the device to work with most MIDI software and DAWs that support MIDI CC learn or mapping.

### Serial (picocom or combined mode)

Once the firmware is running, you can use any terminal program to communicate with the UART and SPI peripherals via USB. The device will appear as a USB CDC (Communications Device Class) device. Currently `/dev/ttyACM0` (macOS: `/dev/tty.usbmodemOSFC20241`) is a debug console that prints information about the Pico's current operation.

### UART Communication (picocom or combined mode)

To communicate with the UART peripheral, open the corresponding serial port (e.g., `/dev/ttyACM1` on Linux, `/dev/tty.usbmodemOSFC20243` on macOS) with your terminal program. For now the Baud is fixed at 115200 but can be changed in code. Dynamic reconfiguration is still planned.

### Using Flashrom or Flashprog (picocom or combined mode)

To interact with the Raspberry Pi Pico for reading and writing SPI flash chips, you can use tools like `flashrom` or `flashprog`. These tools support the `serprog` protocol, which allows communication over a serial interface.

1. Install `flashrom` or `flashprog` e.g.:

```sh
# Debian or Debian-based Linux distributions
sudo apt-get install flashrom

# macOS with Homebrew
brew install flashrom
```

2. Use the following command to read from the SPI flash chip:

```sh
flashrom -p serprog:dev=/dev/ttyACM2 -r backup.bin
```

This command reads the contents of the SPI flash chip and saves it to `backup.bin`.

3. Use the following command to write to the SPI flash chip:

```sh
flashrom -p serprog:dev=/dev/ttyACM2 -w firmware.bin
```

This command writes the contents of `firmware.bin` to the SPI flash chip.

Make sure to replace `/dev/ttyACM2` with the correct serial port if your device is connected to a different port or if you're on a different operating system (on macOS it will be `/dev/tty.usbmodemOSFC20245`).


## License

This project is licensed under the Apache 2.0 License. See the [LICENSE](LICENSE) file for details.
