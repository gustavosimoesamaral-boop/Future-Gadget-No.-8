use x86_64::instructions::port::Port;

pub struct Keyboard {
    pub shift_pressed: bool,
    pub caps_lock: bool,
    pub ctrl_pressed: bool,
    pub extended_scancode: bool,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            shift_pressed: false,
            caps_lock: false,
            ctrl_pressed: false,
            extended_scancode: false,
        }
    }

    pub fn read_scancode(&mut self) -> Option<u8> {
        let mut status_port: Port<u8> = Port::new(0x64);
        let mut data_port: Port<u8> = Port::new(0x60);

        let status: u8 = unsafe { status_port.read() };

        if status & 1 != 0 {
            Some(unsafe { data_port.read() })
        } else {
            None
        }
    }

    pub fn update_modifiers(&mut self, scancode: u8) -> bool {
        // Shift esquerdo/direito pressionado.
        if scancode == 0x2A || scancode == 0x36 {
            self.shift_pressed = true;
            return true;
        }

        // Shift esquerdo/direito solto.
        if scancode == 0xAA || scancode == 0xB6 {
            self.shift_pressed = false;
            return true;
        }

        // Ctrl esquerdo pressionado.
        if scancode == 0x1D {
            self.ctrl_pressed = true;
            return true;
        }

        // Ctrl esquerdo solto.
        if scancode == 0x9D {
            self.ctrl_pressed = false;
            return true;
        }

        // Caps Lock.
        if scancode == 0x3A {
            self.caps_lock = !self.caps_lock;
            return true;
        }

        false
    }

    pub fn scancode_to_ascii(&self, scancode: u8) -> Option<char> {
        let uppercase = self.shift_pressed ^ self.caps_lock;

        match scancode {
            // Números / símbolos.
            0x02 => Some(if self.shift_pressed { '!' } else { '1' }),
            0x03 => Some(if self.shift_pressed { '@' } else { '2' }),
            0x04 => Some(if self.shift_pressed { '#' } else { '3' }),
            0x05 => Some(if self.shift_pressed { '$' } else { '4' }),
            0x06 => Some(if self.shift_pressed { '%' } else { '5' }),
            0x07 => Some(if self.shift_pressed { '^' } else { '6' }),
            0x08 => Some(if self.shift_pressed { '&' } else { '7' }),
            0x09 => Some(if self.shift_pressed { '*' } else { '8' }),
            0x0A => Some(if self.shift_pressed { '(' } else { '9' }),
            0x0B => Some(if self.shift_pressed { ')' } else { '0' }),

            // Letras.
            0x10 => Some(if uppercase { 'Q' } else { 'q' }),
            0x11 => Some(if uppercase { 'W' } else { 'w' }),
            0x12 => Some(if uppercase { 'E' } else { 'e' }),
            0x13 => Some(if uppercase { 'R' } else { 'r' }),
            0x14 => Some(if uppercase { 'T' } else { 't' }),
            0x15 => Some(if uppercase { 'Y' } else { 'y' }),
            0x16 => Some(if uppercase { 'U' } else { 'u' }),
            0x17 => Some(if uppercase { 'I' } else { 'i' }),
            0x18 => Some(if uppercase { 'O' } else { 'o' }),
            0x19 => Some(if uppercase { 'P' } else { 'p' }),

            0x1E => Some(if uppercase { 'A' } else { 'a' }),
            0x1F => Some(if uppercase { 'S' } else { 's' }),
            0x20 => Some(if uppercase { 'D' } else { 'd' }),
            0x21 => Some(if uppercase { 'F' } else { 'f' }),
            0x22 => Some(if uppercase { 'G' } else { 'g' }),
            0x23 => Some(if uppercase { 'H' } else { 'h' }),
            0x24 => Some(if uppercase { 'J' } else { 'j' }),
            0x25 => Some(if uppercase { 'K' } else { 'k' }),
            0x26 => Some(if uppercase { 'L' } else { 'l' }),

            0x2C => Some(if uppercase { 'Z' } else { 'z' }),
            0x2D => Some(if uppercase { 'X' } else { 'x' }),
            0x2E => Some(if uppercase { 'C' } else { 'c' }),
            0x2F => Some(if uppercase { 'V' } else { 'v' }),
            0x30 => Some(if uppercase { 'B' } else { 'b' }),
            0x31 => Some(if uppercase { 'N' } else { 'n' }),
            0x32 => Some(if uppercase { 'M' } else { 'm' }),

            // Espaço.
            0x39 => Some(' '),

            // Símbolos sem Shift.
            0x0C => Some(if self.shift_pressed { '_' } else { '-' }),
            0x0D => Some(if self.shift_pressed { '+' } else { '=' }),
            0x33 => Some(if self.shift_pressed { '<' } else { ',' }),
            0x34 => Some(if self.shift_pressed { '>' } else { '.' }),
            0x35 => Some(if self.shift_pressed { '?' } else { '/' }),

            0x1A => Some(if self.shift_pressed { '{' } else { '[' }),
            0x1B => Some(if self.shift_pressed { '}' } else { ']' }),
            0x27 => Some(if self.shift_pressed { ':' } else { ';' }),
            0x28 => Some(if self.shift_pressed { '"' } else { '\'' }),
            0x29 => Some(if self.shift_pressed { '~' } else { '`' }),

            _ => None,
        }
    }

    pub fn is_extended(&mut self, scancode: u8) -> bool {
        if scancode == 0xE0 {
            self.extended_scancode = true;
            return true;
        }

        false
    }

    pub fn take_extended(&mut self, scancode: u8) -> Option<u8> {
        if !self.extended_scancode {
            return None;
        }

        self.extended_scancode = false;

        // Tecla especial sendo solta.
        if scancode & 0x80 != 0 {
            return None;
        }

        Some(scancode)
    }
}