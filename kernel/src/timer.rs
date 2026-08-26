use x86_64::instructions::port::Port;

// Frequência do PIT:
// 1.193.182 Hz / 11.932 ≈ 100 Hz
pub const TIMER_FREQUENCY: u64 = 100;

pub fn init() {
    let divisor: u16 = 11_932;

    let mut command_port: Port<u8> = Port::new(0x43);
    let mut channel0: Port<u8> = Port::new(0x40);

    unsafe {
        // Canal 0
        // Acesso low byte + high byte
        // Modo 2: Rate Generator
        // Base binária
        command_port.write(0x34);

        channel0.write((divisor & 0xFF) as u8);
        channel0.write((divisor >> 8) as u8);
    }
}