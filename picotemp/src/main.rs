//! Raspberry Pi Pico - LED Controller with Morse Code
#![no_std]
#![no_main]

use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal::{
    clocks::init_clocks_and_plls,
    pac,
    sio::Sio,
    usb::UsbBus,
    watchdog::Watchdog,
    Timer,
};

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::delay::DelayMs;

use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;
use core::fmt::Write;

#[derive(Clone, Copy)]
enum Pattern {
    Off,
    Solid,
    Blink,
    FastBlink,
    Pulse,
    Strobe,
    Morse,
}

struct MorseEncoder {
    message: heapless::Vec<u8, 256>,
    position: usize,
    dot_length_ms: u32,
}

impl MorseEncoder {
    fn new() -> Self {
        Self {
            message: heapless::Vec::new(),
            position: 0,
            dot_length_ms: 200, // Default dot length
        }
    }

    fn set_message(&mut self, msg: &[u8]) {
        self.message.clear();
        for &byte in msg {
            let _ = self.message.push(byte);
        }
        self.position = 0;
    }

    fn set_speed(&mut self, wpm: u32) {
        // Words per minute to dot length
        // Standard: PARIS = 50 dot lengths
        // WPM = (dot_length * 50) / 60000
        self.dot_length_ms = 1200 / wpm.max(5).min(40);
    }

    fn char_to_morse(c: char) -> Option<&'static str> {
        match c.to_ascii_uppercase() {
            'A' => Some(".-"),
            'B' => Some("-..."),
            'C' => Some("-.-."),
            'D' => Some("-.."),
            'E' => Some("."),
            'F' => Some("..-."),
            'G' => Some("--."),
            'H' => Some("...."),
            'I' => Some(".."),
            'J' => Some(".---"),
            'K' => Some("-.-"),
            'L' => Some(".-.."),
            'M' => Some("--"),
            'N' => Some("-."),
            'O' => Some("---"),
            'P' => Some(".--."),
            'Q' => Some("--.-"),
            'R' => Some(".-."),
            'S' => Some("..."),
            'T' => Some("-"),
            'U' => Some("..-"),
            'V' => Some("...-"),
            'W' => Some(".--"),
            'X' => Some("-..-"),
            'Y' => Some("-.--"),
            'Z' => Some("--.."),
            '0' => Some("-----"),
            '1' => Some(".----"),
            '2' => Some("..---"),
            '3' => Some("...--"),
            '4' => Some("....-"),
            '5' => Some("....."),
            '6' => Some("-...."),
            '7' => Some("--..."),
            '8' => Some("---.."),
            '9' => Some("----."),
            '.' => Some(".-.-.-"),
            ',' => Some("--..--"),
            '?' => Some("..--.."),
            '!' => Some("-.-.--"),
            ' ' => Some(" "),
            _ => None,
        }
    }

    fn play_morse<P: OutputPin>(&mut self, led: &mut P, timer: &mut Timer) -> bool {
        if self.position >= self.message.len() {
            return true; // Done
        }

        let c = self.message[self.position] as char;
        
        if let Some(morse) = Self::char_to_morse(c) {
            for symbol in morse.chars() {
                match symbol {
                    '.' => {
                        // Dot: LED on for 1 unit
                        let _ = led.set_high();
                        timer.delay_ms(self.dot_length_ms);
                        let _ = led.set_low();
                        timer.delay_ms(self.dot_length_ms); // Space between symbols
                    }
                    '-' => {
                        // Dash: LED on for 3 units
                        let _ = led.set_high();
                        timer.delay_ms(self.dot_length_ms * 3);
                        let _ = led.set_low();
                        timer.delay_ms(self.dot_length_ms); // Space between symbols
                    }
                    ' ' => {
                        // Word space: 7 units (already have 1 from last char, so add 6)
                        timer.delay_ms(self.dot_length_ms * 6);
                    }
                    _ => {}
                }
            }
            // Space between letters: 3 units (already have 1, so add 2)
            timer.delay_ms(self.dot_length_ms * 2);
        }

        self.position += 1;
        false // Not done yet
    }

    fn is_done(&self) -> bool {
        self.position >= self.message.len()
    }

    fn reset(&mut self) {
        self.position = 0;
    }
}

struct LedController {
    pattern: Pattern,
    speed_ms: u32,
    counter: u32,
    morse: MorseEncoder,
}

impl LedController {
    fn new() -> Self {
        Self {
            pattern: Pattern::Blink,
            speed_ms: 500,
            counter: 0,
            morse: MorseEncoder::new(),
        }
    }

    fn update<P: OutputPin>(&mut self, led: &mut P, timer: &mut Timer) {
        self.counter = self.counter.wrapping_add(1);
        
        match self.pattern {
            Pattern::Off => {
                let _ = led.set_low();
            }
            Pattern::Solid => {
                let _ = led.set_high();
            }
            Pattern::Blink => {
                if self.counter % (self.speed_ms / 10) == 0 {
                    let _ = led.set_high();
                    timer.delay_ms(50u32);
                    let _ = led.set_low();
                }
            }
            Pattern::FastBlink => {
                if self.counter % 10 == 0 {
                    let _ = led.set_high();
                    timer.delay_ms(50u32);
                    let _ = led.set_low();
                }
            }
            Pattern::Pulse => {
                let cycle = self.speed_ms / 10;
                if self.counter % (cycle * 2) < cycle {
                    let _ = led.set_high();
                } else {
                    let _ = led.set_low();
                }
            }
            Pattern::Strobe => {
                if self.counter % 5 == 0 {
                    let _ = led.set_high();
                    timer.delay_ms(10u32);
                    let _ = led.set_low();
                }
            }
            Pattern::Morse => {
                // Play morse code message
                if self.morse.is_done() {
                    // Loop the message
                    self.morse.reset();
                    timer.delay_ms(self.morse.dot_length_ms * 7); // Word space before repeat
                }
                let _ = self.morse.play_morse(led, timer);
                return; // Don't do the normal delay
            }
        }
    }

    fn set_pattern(&mut self, pattern: Pattern) {
        self.pattern = pattern;
        self.counter = 0;
    }

    fn set_speed(&mut self, speed_ms: u32) {
        self.speed_ms = speed_ms.max(50).min(5000);
    }

    fn set_morse_message(&mut self, msg: &[u8]) {
        self.morse.set_message(msg);
        self.pattern = Pattern::Morse;
    }

    fn set_morse_speed(&mut self, wpm: u32) {
        self.morse.set_speed(wpm);
    }
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    
    let mut led_pin = pins.led.into_push_pull_output();
    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    
    let mut serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x2e8a, 0x000a))
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    let mut led_controller = LedController::new();
    let mut input_buffer: heapless::Vec<u8, 128> = heapless::Vec::new();
    let mut welcome_sent = false;

    loop {
        if usb_dev.poll(&mut [&mut serial]) {
            if !welcome_sent {
                let welcome = b"\r\n=== Pico LED Controller with Morse Code ===\r\nType 'help' for commands\r\n> ";
                let _ = serial.write(welcome);
                welcome_sent = true;
            }

            let mut buf = [0u8; 64];
            match serial.read(&mut buf) {
                Ok(count) if count > 0 => {
                    for &byte in &buf[0..count] {
                        let _ = serial.write(&[byte]);
                        
                        if byte == b'\r' || byte == b'\n' {
                            if input_buffer.len() > 0 {
                                let response = process_command(&input_buffer, &mut led_controller);
                                let _ = serial.write(b"\r\n");
                                let _ = serial.write(response.as_bytes());
                                let _ = serial.write(b"\r\n> ");
                                input_buffer.clear();
                            } else {
                                let _ = serial.write(b"\r\n> ");
                            }
                        } else if byte == 8 || byte == 127 {
                            if input_buffer.len() > 0 {
                                input_buffer.pop();
                                let _ = serial.write(b"\x08 \x08");
                            }
                        } else if byte >= 32 && byte < 127 {
                            let _ = input_buffer.push(byte);
                        }
                    }
                }
                _ => {}
            }
        }

        led_controller.update(&mut led_pin, &mut timer);
        timer.delay_ms(10u32);
    }
}

fn process_command(cmd_bytes: &[u8], controller: &mut LedController) -> heapless::String<256> {
    let mut response: heapless::String<256> = heapless::String::new();
    
    let cmd_str = core::str::from_utf8(cmd_bytes).unwrap_or("").trim();
    let parts: heapless::Vec<&str, 8> = cmd_str.split_whitespace().collect();
    
    if parts.is_empty() {
        return response;
    }

    match parts[0] {
        "help" | "h" => {
            let _ = write!(&mut response, 
                "Commands:\r\n\
                 solid on/off  - Turn LED solid on or off\r\n\
                 blink         - Normal blink pattern\r\n\
                 fast          - Fast blink pattern\r\n\
                 pulse         - Slow pulse pattern\r\n\
                 strobe        - Strobe effect\r\n\
                 morse <text>  - Blink message in morse code\r\n\
                 wpm <speed>   - Set morse speed (5-40 WPM)\r\n\
                 speed <ms>    - Set blink speed (50-5000ms)\r\n\
                 status        - Show current settings\r\n\
                 help          - Show this help");
        }
        "solid" => {
            if parts.len() > 1 {
                match parts[1] {
                    "on" => {
                        controller.set_pattern(Pattern::Solid);
                        let _ = write!(&mut response, "LED solid ON");
                    }
                    "off" => {
                        controller.set_pattern(Pattern::Off);
                        let _ = write!(&mut response, "LED OFF");
                    }
                    _ => {
                        let _ = write!(&mut response, "Usage: solid on/off");
                    }
                }
            } else {
                let _ = write!(&mut response, "Usage: solid on/off");
            }
        }
        "blink" | "b" => {
            controller.set_pattern(Pattern::Blink);
            let _ = write!(&mut response, "Blink pattern set ({}ms)", controller.speed_ms);
        }
        "fast" | "f" => {
            controller.set_pattern(Pattern::FastBlink);
            let _ = write!(&mut response, "Fast blink pattern set");
        }
        "pulse" | "p" => {
            controller.set_pattern(Pattern::Pulse);
            let _ = write!(&mut response, "Pulse pattern set ({}ms)", controller.speed_ms);
        }
        "strobe" | "s" => {
            controller.set_pattern(Pattern::Strobe);
            let _ = write!(&mut response, "Strobe pattern set");
        }
        "morse" | "m" => {
            if parts.len() > 1 {
                // Join all parts after "morse" to get the full message
                let message = &cmd_str[parts[0].len()..].trim();
                controller.set_morse_message(message.as_bytes());
                let _ = write!(&mut response, "Morse code set: '{}'", message);
            } else {
                let _ = write!(&mut response, "Usage: morse <text>");
            }
        }
        "wpm" => {
            if parts.len() > 1 {
                if let Ok(wpm) = parts[1].parse::<u32>() {
                    controller.set_morse_speed(wpm);
                    let _ = write!(&mut response, "Morse speed set to {} WPM", wpm);
                } else {
                    let _ = write!(&mut response, "Invalid WPM. Use: wpm <number>");
                }
            } else {
                let _ = write!(&mut response, "Usage: wpm <5-40>");
            }
        }
        "speed" => {
            if parts.len() > 1 {
                if let Ok(speed) = parts[1].parse::<u32>() {
                    controller.set_speed(speed);
                    let _ = write!(&mut response, "Speed set to {}ms", controller.speed_ms);
                } else {
                    let _ = write!(&mut response, "Invalid speed. Use: speed <ms>");
                }
            } else {
                let _ = write!(&mut response, "Usage: speed <ms> (50-5000)");
            }
        }
        "status" => {
            let pattern_name = match controller.pattern {
                Pattern::Off => "Off",
                Pattern::Solid => "Solid",
                Pattern::Blink => "Blink",
                Pattern::FastBlink => "Fast Blink",
                Pattern::Pulse => "Pulse",
                Pattern::Strobe => "Strobe",
                Pattern::Morse => "Morse Code",
            };
            let _ = write!(&mut response, "Pattern: {}, Speed: {}ms", pattern_name, controller.speed_ms);
        }
        _ => {
            let _ = write!(&mut response, "Unknown command: '{}'. Type 'help' for commands", parts[0]);
        }
    }

    response
}
