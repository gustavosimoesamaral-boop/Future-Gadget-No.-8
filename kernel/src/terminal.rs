use crate::reboot;
use bootloader_api::info::FrameBufferInfo;
use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight};

pub struct Terminal<'a> {
    buffer: &'a mut [u8],
    info: FrameBufferInfo,
    cursor_x: usize,
    cursor_y: usize,

    command: [u8; 64],
    command_len: usize,
    command_cursor: usize,
    pub command_selected: bool,

    uptime_seconds: u64,

    history: [[u8; 64]; 8],
    history_len: [usize; 8],
    pub history_count: usize,
    pub history_index: usize,
}

impl<'a> Terminal<'a> {
    pub fn new(
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

            uptime_seconds: 0,

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

        pub fn update_uptime(&mut self, seconds: u64) {
            self.uptime_seconds = seconds;
        }

        fn write_num(&mut self, mut number: u64) {
            if number == 0 {
                self.put_char('0');
                return;
            }

            let mut digits = [0u8; 20];
            let mut len = 0;

            while number > 0 {
                digits[len] = (number % 10) as u8;
                number /= 10;
                len += 1;
            }

            while len > 0 {
                len -= 1;
                self.put_char((b'0' + digits[len]) as char);
            }
        }

        pub fn select_all(&mut self) {
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
        
        pub fn load_history(&mut self, index: usize) {
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

        pub fn push_command_char(&mut self, c: char) {
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

        pub fn remove_command_char(&mut self) {
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

        pub fn clear_command(&mut self) {
            self.command = [0; 64];
            self.command_len = 0;
            self.command_cursor = 0;
        }

        pub fn execute_command(&mut self) {
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
                self.write("  uptime - mostra o tempo ligado do sistema\n")
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

            else if command == b"uptime" {
                let total_seconds = self.uptime_seconds;

                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;

                self.write("\n\nNOSSO_OS uptime: ");

                // Horas
                if hours < 10 {
                    self.write("0");
                }

                self.write_num(hours);
                self.write(":");

                // Minutos
                if minutes < 10 {
                    self.write("0");
                }

                self.write_num(minutes);
                self.write(":");

                // Segundos
                if seconds < 10 {
                    self.write("0");
                }

                self.write_num(seconds);
            }

            else {
                self.write("\n\nComando desconhecido.\n");
                self.write("Digite 'help' para ver os comandos.\n");
            }

            self.write("\n> ");
            self.clear_command();
        }

        pub fn clear(&mut self) {
            for byte in self.buffer.iter_mut() {
                *byte = 0;
            }

            self.cursor_x = 32;
            self.cursor_y = 32;
        }

        pub fn write(&mut self, text: &str) {
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

        pub fn draw_cursor(&mut self, visible: bool) {
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

        pub fn redraw_command(&mut self) {
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

        pub fn cancel_command(&mut self) {
            self.clear_command();
            self.command_selected = false;

            self.redraw_command();
        }

        pub fn move_cursor_left(&mut self) {
            if self.command_cursor > 0 {
                self.command_cursor -= 1;
                self.redraw_command();
            }
        }

        pub fn move_cursor_right(&mut self) {
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