use x86_64::instructions::port::Port;

pub struct PitTimer {
    last_count: u16,
    ticks: u64,
}

impl PitTimer {
    pub fn new() -> Self {
        let divisor: u16 = 11_932;

        let mut command_port: Port<u8> = Port::new(0x43);
        let mut channel0: Port<u8> = Port::new(0x40);

        unsafe {
            command_port.write(0x34);

            channel0.write((divisor & 0xFF) as u8);
            channel0.write((divisor >> 8) as u8);
        }

        Self {
            last_count: 0,
            ticks: 0,
        }
    }

    pub fn poll(&mut self) {
        let mut command_port: Port<u8> = Port::new(0x43);
        let mut channel0: Port<u8> = Port::new(0x40);

        unsafe {
            command_port.write(0x00);

            let low = channel0.read();
            let high = channel0.read();

            let current_count =
                u16::from(low) | (u16::from(high) << 8);

            if self.last_count != 0
                && current_count > self.last_count
            {
                self.ticks += 1;
            }

            self.last_count = current_count;
        }
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    pub fn seconds(&self) -> u64 {
        self.ticks / 100
    }
}