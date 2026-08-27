#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupts;
mod keyboard;
mod terminal;
mod timer;

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use keyboard::Keyboard;
use terminal::Terminal;
use x86_64::instructions::port::Port;

entry_point!(kernel_main);

pub fn reboot() -> ! {
    let mut command_port: Port<u8> = Port::new(0x64);

    // Espera o controlador ficar pronto para receber um comando.
    loop {
        let status: u8 = unsafe { command_port.read() };

        if status & 0x02 == 0 {
            break;
        }
    }

    // Comando 0xFE = reset do sistema através do controlador 8042.
    unsafe {
        command_port.write(0xFE);
    }

    // Se o hardware/emulador não reiniciar por algum motivo,
    // ficamos parados aqui.
    loop {
        x86_64::instructions::hlt();
    }
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    timer::init();
    interrupts::init();
    interrupts::enable();

    let mut last_second = 0;
    let mut cursor_visible = true;
    let mut keyboard = Keyboard::new();

    let framebuffer = boot_info
        .framebuffer
        .as_mut()
        .expect("Framebuffer nao encontrado");

    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();

    let mut terminal = Terminal::new(buffer, info);

    terminal.draw_cursor(true);

    loop {

        let current_second = 
            interrupts::timer_ticks() / timer::TIMER_FREQUENCY;

        if current_second != last_second {
            last_second = current_second;

            terminal.update_uptime(current_second);

            cursor_visible = !cursor_visible;
            terminal.draw_cursor(cursor_visible);
        }

        let Some(scancode) = 
            interrupts::keyboard_scancode() else {
            continue;
        };

        if keyboard.update_modifiers(scancode) {
            cursor_visible = true;
            terminal.draw_cursor(true);
            continue;
        }

        if keyboard.is_extended(scancode) {
            continue;
        }

        // Tecla sendo solta.
        if scancode & 0x80 != 0 {
            terminal.draw_cursor(cursor_visible);
            continue;
        }

        // Esconde o cursor antes de alterar o terminal.
        terminal.draw_cursor(false);

        // Ctrl + A
        if keyboard.ctrl_pressed && scancode == 0x1E {
            terminal.select_all();

            cursor_visible = true;
            terminal.draw_cursor(true);
            continue;
        }

        // Ctrl + C
        if keyboard.ctrl_pressed && scancode == 0x2E {
            terminal.cancel_command();

            cursor_visible = true;
            terminal.draw_cursor(true);
            continue;
        }

        // Ctrl + L
        if keyboard.ctrl_pressed && scancode == 0x26 {
            terminal.clear();

            terminal.write("NOSSO_OS\n\n");
            terminal.write("Kernel iniciado com sucesso!\n\n");
            terminal.write("> ");

            terminal.clear_command();
            terminal.command_selected = false;

            cursor_visible = true;
            terminal.draw_cursor(true);
            continue;
        }

        // Teclas especiais do teclado.
        if let Some(extended_scancode) = keyboard.take_extended(scancode) {
            match extended_scancode {
                // ↑
                0x48 => {
                    if terminal.history_count > 0 {
                        if terminal.history_index > 0 {
                            terminal.history_index -= 1;
                        }

                        terminal.load_history(terminal.history_index);
                    }
                }

                // ↓
                0x50 => {
                    if terminal.history_count > 0 {
                        if terminal.history_index < terminal.history_count {
                            terminal.history_index += 1;
                        }

                        if terminal.history_index < terminal.history_count {
                            terminal.load_history(terminal.history_index);
                        } else {
                            terminal.clear_command();
                            terminal.redraw_command();
                        }
                    }
                }

                // ←
                0x4B => {
                    terminal.move_cursor_left();
                }

                // →
                0x4D => {
                    terminal.move_cursor_right();
                }

                _ => {}
            }

            cursor_visible = true;
            terminal.draw_cursor(true);
            continue;
        }

        // Enter.
        if scancode == 0x1C {
            terminal.execute_command();
        }

        // Backspace.
        else if scancode == 0x0E {
            terminal.remove_command_char();
        }

        // Caractere normal.
        else if let Some(character) = 
            keyboard.scancode_to_ascii(scancode) 
        {
            terminal.push_command_char(character);
        }

        // Cursor reaparece na posição nova.
        cursor_visible = true;
        terminal.draw_cursor(true);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}