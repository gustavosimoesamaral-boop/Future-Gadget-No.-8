#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight};
use x86_64::instructions::port::Port;

entry_point!(kernel_main);


struct Terminal<'a> {
    buffer: &'a mut [u8],
    info: bootloader_api::info::FrameBufferInfo,
    cursor_x: usize,
    cursor_y: usize,

    command: [u8; 64],
    command_len: usize,
    command_cursor: usize,
    command_selected: bool,

    history: [[u8; 64]; 8],
    history_len: [usize; 8],
    history_count: usize,
    history_index: usize,
}

impl<'a> Terminal<'a> {
    fn new(
        buffer: &'a mut [u8],
        info: bootloader_api::info::FrameBufferInfo,
    ) -> Self {

        let mut terminal = Self {
            buffer,
            info,
            cursor_x: 32,
            cursor_y: 32,

            command: [0; 64],
            command_len: 0,
            command_cursor: 0,
            command_selected: false,

            history: [[0; 64]; 8],
            history_len: [0; 8],
            history_count: 0,
            history_index: 0,
        };

        terminal.clear();
        terminal.write("NOSSO_OS\n\n");
        terminal.write("Kernel iniciado com sucesso!\n\n");
        terminal.write("> ");

        terminal
    }

        fn select_all(&mut self) {
            if self.command_len == 0 {
                return;
            }

            self.command_selected = true;

            // A seleção visual será implementada depois.
            self.command_cursor = self.command_len;

            self.redraw_command();
        }
        
        fn save_history(&mut self) {
            if self.command_len == 0 {
                return;
            }

            if self.history_count < 8 {
                self.history[self.history_count] = [0; 64];

                self.history[self.history_count][..self.command_len]
                    .copy_from_slice(&self.command[..self.command_len]);

                self.history_len[self.history_count] = self.command_len;
                self.history_count += 1;
            } else {
                // Remove o comando mais antigo.
                for i in 0..7 {
                    self.history[i] = self.history[i + 1];
                    self.history_len[i] = self.history_len[i + 1];
                }

                // Coloca o novo comando no final.
                self.history[7] = [0; 64];

                self.history[7][..self.command_len]
                    .copy_from_slice(&self.command[..self.command_len]);

                self.history_len[7] = self.command_len;
            }

            // Começa depois do comando mais recente.
            self.history_index = self.history_count;
        }
        
        fn load_history(&mut self, index: usize) {
            if index >= self.history_count {
                return;
            }
            
            // Apaga o comando atual da tela.
            let current_len = self.command_len;

            for _ in 0..current_len {
                self.backspace();
            }

            // Carrega o comando escolhido.
            self.command = [0; 64];

            let len = self.history_len[index];

            self.command[..len]
                .copy_from_slice(&self.history[index][..len]);

            self.command_len = len;
            self.command_cursor = self.command_len;
            self.command_selected = false;

            self.redraw_command();

            // Redesenha o comando.
            self.redraw_command();
        }

        fn push_command_char(&mut self, c: char) {
            if !c.is_ascii() {
                return;
            }

            if self.command_selected {
                self.command = [0; 64];
                self.command_len = 0;
                self.command_cursor = 0;
                self.command_selected = false;
            }

            if self.command_len >= self.command.len() {
                return;
            }

            // Move o texto para abrir espaço no cursor.
            for i in (self.command_cursor..self.command_len).rev() {
                self.command[i + 1] = self.command[i];
            }

            self.command[self.command_cursor] = c as u8;
            self.command_len += 1;
            self.command_cursor += 1;

            self.redraw_command();
        }    

        fn remove_command_char(&mut self) {
            if self.command_selected {
                self.clear_command();
                self.command_selected = false;
                self.redraw_command();
                return;
            }

            if self.command_cursor == 0 {
                return;
            }

            // Remove o caractere imediatamente na esquerda do cursor.
            for i in self.command_cursor..self.command_len {
                self.command[i - 1] = self.command[i];
            }

            self.command_len -= 1;
            self.command[self.command_len] = 0;

            self.command_cursor -= 1;

            // Redesenha a linha já com o caractere removido.
            self.redraw_command();
        }

        fn clear_command(&mut self) {
            self.command = [0; 64];
            self.command_len = 0;
            self.command_cursor = 0;
        }

        fn execute_command(&mut self) {
            let command_len = self.command_len;

            let mut command = [0u8; 64];
            command[..command_len].copy_from_slice(&self.command[..command_len]);

            if command_len == 0 {
                self.write("\n> ");
                return;
            }

            let command = &command[..command_len];

            self.save_history();

            if command == b"help" {
                self.write("\n\nComandos disponiveis:\n");
                self.write("  help   - mostra esta mensagem\n");
                self.write("  about  - mostra informacoes do sistema\n");
                self.write("  clear  - limpa a tela\n");
                self.write("  echo   - mostra um texto\n");
                self.write("  reboot - reinicia o sistema\n");
            }

            else if command == b"about" {
                self.write("\n\nNOSSO_OS\n");
                self.write("Gustavo Corporation\n\n");
                self.write("Architecture : x86_64\n");
                self.write("Kernel       : 0.1.0\n");
                self.write("Status       : ONLINE\n");
            }

            else if command == b"clear" {
                self.clear();

                self.write("NOSSO_OS\n\n");
                self.write("Kernel iniciado com sucesso!\n\n");
                self.write("> ");

                self.clear_command();
                return;
            }

            else if command == b"echo" {
                self.write("\n\n");
                self.write("\n> ");
            }
    
            else if command.starts_with(b"echo ") {
                self.write("\n");

                for &byte in &command[5..] {
                    self.put_char(byte as char);
                }

                self.write("\n");
            }

            else if command == b"reboot" {
                self.write("\n\nReiniciando o NOSSO_OS...\n");
                reboot();
            }

            else {
                self.write("\n\nComando desconhecido.\n");
                self.write("Digite 'help' para ver os comandos.\n");
            }

            self.write("\n> ");
            self.clear_command();
        }

        fn clear(&mut self) {
            for byte in self.buffer.iter_mut() {
                *byte = 0;
            }

            self.cursor_x = 32;
            self.cursor_y = 32;
        }

        fn write(&mut self, text: &str) {
            for c in text.chars() {
                self.put_char(c);
            }
        }

        fn blend_pixel(
            &mut self,
            px: usize,
            py: usize,
            red: u8,
            green: u8,
            blue: u8,
            alpha: u8,
        ) {
            if px >= self.info.width || py >= self.info.height {
                return;
            }

            let offset =
                py * self.info.stride * self.info.bytes_per_pixel
                    + px * self.info.bytes_per_pixel;

            if offset + 2 >= self.buffer.len() {
                return;
            }

            let old_red = self.buffer[offset] as u16;
            let old_green = self.buffer[offset + 1] as u16;
            let old_blue = self.buffer[offset + 2] as u16;

            let alpha = alpha as u16;
            let inverse_alpha = 255 - alpha;

            self.buffer[offset] =
                ((red as u16 * alpha + old_red * inverse_alpha) / 255) as u8;

            self.buffer[offset + 1] =
                ((green as u16 * alpha + old_green * inverse_alpha) / 255) as u8;

            self.buffer[offset + 2] =
                ((blue as u16 * alpha + old_blue * inverse_alpha) / 255) as u8;
        }

        fn draw_cursor(&mut self, visible: bool) {
            let width = 8;
            let height = 16;

            for row in 0..height {
                for column in 0..width {
                    let px = self.cursor_x + column;
                    let py = self.cursor_y + row;

                    if px >= self.info.width || py >= self.info.height {
                        continue;
                    }

                    let offset =
                    py * self.info.stride * self.info.bytes_per_pixel
                    + px * self.info.bytes_per_pixel;

                    if offset + 2 >= self.buffer.len() {
                        continue;
                    }

                    let value = if visible { 255 } else { 0 };

                    self.buffer[offset] = value;
                    self.buffer[offset + 1] = value;
                    self.buffer[offset + 2] = value;
                }
            }
        }

        fn redraw_command(&mut self) {
            // Volta para o início da linha do comando.
            self.cursor_x = 52;

            // Apaga toda a área onde o comando poderia estar.
            for row in 0..16 {
                for column in 0..(64 * 10) {
                    let px = self.cursor_x + column;
                    let py = self.cursor_y + row;

                    if px >= self.info.width || py >= self.info.height {
                        continue;
                    }

                    let offset =
                    py * self.info.stride * self.info.bytes_per_pixel
                    + px * self.info.bytes_per_pixel;

                    if offset + 2 >= self.buffer.len() {
                        continue;
                    }

                    self.buffer[offset] = 0;
                    self.buffer[offset + 1] = 0;
                    self.buffer[offset + 2] = 0;
                }
            }

            // Redesenha o comando inteiro.
            let command = self.command;
            let command_len = self.command_len;

            if self.command_selected {
                self.draw_selection();
            }

            for i in 0..command_len {
                self.put_char(command[i] as char);
            }

            // Reposiciona o cursor lógico/visual.
            self.cursor_x = 52;

            
            for i in 0..self.command_cursor {
                let Some(raster) = get_raster(
                    self.command[i] as char,
                    FontWeight::Regular,
                    RasterHeight::Size16,
                ) else {
                    continue;
                };

                self.cursor_x += raster.width() + 2;
            }
        }

        fn cancel_command(&mut self) {
            self.clear_command();
            self.command_selected = false;

            self.redraw_command();
        }

        fn move_cursor_left(&mut self) {
            if self.command_cursor > 0 {
                self.command_cursor -= 1;
                self.redraw_command();
            }
        }

        fn move_cursor_right(&mut self) {
            if self.command_cursor < self.command_len {
                self.command_cursor += 1;
                self.redraw_command();
            }
        }

        fn draw_selection(&mut self) {
            if !self.command_selected || self.command_len == 0 {
                return;
            }

            let start_x = 52;
            let mut end_x = start_x;

            let command = self.command;

            for i in 0..self.command_len {
                let Some(raster) = get_raster(
                    command[i] as char,
                    FontWeight::Regular,
                    RasterHeight::Size16,
                ) else {
                    continue;
                };

                end_x += raster.width() + 2;
            }

            // Azul com transparência.
            let red = 40;
            let green = 120;
            let blue = 255;
            let alpha = 90;

            for row in 0..18 {
                for px in start_x..end_x {
                    let py = self.cursor_y + row;

                    self.blend_pixel(
                        px,
                        py,
                        red,
                        green,
                        blue,
                        alpha,
                    );
                }
            }
        }

        fn put_char(&mut self, c: char) {

            if c == '\n' {
                self.cursor_x = 32;
                self.cursor_y += 20;
                return;
            }

            if c == '\r' {
                self.cursor_x = 32;
                return;
            }

            if c == '\u{8}' {
                self.backspace();
                return;
            }

            let Some(raster) = get_raster(
                c,
                FontWeight::Regular,
                RasterHeight::Size16,
            ) else {
                return;
            };

            for (row, pixels) in raster.raster().iter().enumerate() {
                for (column, intensity) in pixels.iter().enumerate() {
                    if *intensity == 0 {
                        continue;
                    }

                    let px = self.cursor_x + column;
                    let py = self.cursor_y + row;

                    if px >= self.info.width || py >= self.info.height {
                        continue;
                    }

                    let offset =
                        py * self.info.stride * self.info.bytes_per_pixel
                        + px * self.info.bytes_per_pixel;

                    if offset + 2 >= self.buffer.len() {
                        continue;
                    }

                    self.buffer[offset] = *intensity;
                    self.buffer[offset + 1] = *intensity;
                    self.buffer[offset + 2] = *intensity;
                }
            }

        self.cursor_x += raster.width() + 2;

        // Quebra de linha automática.
        if self.cursor_x + 20 >= self.info.width {
            self.cursor_x = 32;
            self.cursor_y += 20;
        }

    }

    fn backspace(&mut self) {
        self.remove_command_char();
    }
}

// Lê um scancode do teclado PS/2.
// 0x64 = status do controlador
// 0x60 = dados do teclado
fn read_scancode() -> Option<u8> {
    let mut status_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    let status: u8 = unsafe { status_port.read() };

    if status & 1 != 0 {
        Some(unsafe { data_port.read() })
    } else {
        None
    }
}

// Converte scancodes básicos do teclado US para ASCII.
fn scancode_to_ascii(
    scancode: u8,
    shift_pressed: bool,
    caps_lock: bool,
) -> Option<char> {
    let uppercase = shift_pressed ^ caps_lock;

    match scancode {
        0x02 => Some(if shift_pressed { '!' } else { '1' }),
        0x03 => Some(if shift_pressed { '@' } else { '2' }),
        0x04 => Some(if shift_pressed { '#' } else { '3' }),
        0x05 => Some(if shift_pressed { '$' } else { '4' }),
        0x06 => Some(if shift_pressed { '%' } else { '5' }),
        0x07 => Some(if shift_pressed { '^' } else { '6' }),
        0x08 => Some(if shift_pressed { '&' } else { '7' }),
        0x09 => Some(if shift_pressed { '*' } else { '8' }),
        0x0A => Some(if shift_pressed { '(' } else { '9' }),
        0x0B => Some(if shift_pressed { ')' } else { '0' }),

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

        0x39 => Some(' '),

        0x0C => Some('-'),
        0x0D => Some('='),
        0x33 => Some(','),
        0x34 => Some('.'),
        0x35 => Some('/'),

        0x1A => Some('['),
        0x1B => Some(']'),
        0x27 => Some(';'),
        0x28 => Some('\''),
        0x29 => Some('`'),

        _ => None,
    }
}

fn reboot() -> ! {
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
    let framebuffer = boot_info
        .framebuffer
        .as_mut()
        .expect("Framebuffer nao encontrado");

    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();

    let mut terminal = Terminal::new(buffer, info);

    let mut blink_counter: u64 = 0;
    let mut cursor_visible = true;
    let mut extended_scancode = false;
    let mut shift_pressed = false;
    let mut caps_lock = false;
    let mut ctrl_pressed = false;

    terminal.draw_cursor(true);

    loop {
        blink_counter += 1;

        if blink_counter >= 5_000_000 {
            cursor_visible = !cursor_visible;
            terminal.draw_cursor(cursor_visible);
            blink_counter = 0;
        }

        let Some(scancode) = read_scancode() else {
            continue;
        };

        if scancode == 0xE0 {
            extended_scancode = true;
            continue;
        }

        // Esconde o cursor antes de alterar o terminal.
        terminal.draw_cursor(false);

        // Caps Lock.
        if scancode == 0x3A {
            caps_lock = !caps_lock;

            cursor_visible = true;
            terminal.draw_cursor(true);
            continue;
        }

        // Shift esquerdo/direito pressionado.
        if scancode == 0x2A || scancode == 0x36 {
            shift_pressed = true;
            cursor_visible = true;
            terminal.draw_cursor(true);
            continue;
        }

        // Shift esquerdo/direito solto.
        if scancode == 0xAA || scancode == 0xB6 {
            shift_pressed = false;
            cursor_visible = true;
            terminal.draw_cursor(true);
            continue;
        }

        // Ctrl esquerdo pressionado.
        if scancode == 0x1D {
            ctrl_pressed = true;

            cursor_visible = true;
            terminal.draw_cursor(true);
            continue;
        }

        // Ctrl esquerdo solto.
        if scancode == 0x9D {
            ctrl_pressed = false;

            cursor_visible = true;
            terminal.draw_cursor(true);
            continue;
        }

        // Ctrl + A
        if ctrl_pressed && scancode == 0x1E {
            terminal.select_all();

            cursor_visible = true;
            terminal.draw_cursor(true);
            continue;
        }

        // Ctrl + C
        if ctrl_pressed && scancode == 0x2E {
            terminal.cancel_command();

            cursor_visible = true;
            terminal.draw_cursor(true);
            continue;
        }

        // Ctrl + L
        if ctrl_pressed && scancode == 0x26 {
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
        if extended_scancode {
            extended_scancode = false;

            match scancode {
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

        // Tecla sendo solta.
        if scancode & 0x80 != 0 {
            terminal.draw_cursor(cursor_visible);
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
            scancode_to_ascii(scancode, shift_pressed, caps_lock) 
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