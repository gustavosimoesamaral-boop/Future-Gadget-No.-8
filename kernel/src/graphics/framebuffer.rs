use bootloader_api::info::{FrameBufferInfo, PixelFormat};

use super::backend::GraphicsBackend;
use super::types::{Color, Point, Rect};

pub struct Framebuffer<'a> {
    buffer: &'a mut [u8],
    info: FrameBufferInfo,
}

impl<'a> Framebuffer<'a> {
    pub fn new(
        buffer: &'a mut [u8],
        info: FrameBufferInfo,
    ) -> Self {
        Self {
            buffer,
            info,
        }
    }

    pub fn info(&self) -> FrameBufferInfo {
        self.info
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        self.buffer
    }

    pub fn into_buffer(self) -> &'a mut [u8] {
        self.buffer
    }
}

impl<'a> GraphicsBackend for Framebuffer<'a> {
    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }

    fn clear(&mut self, color: Color) {
        self.fill_rect(
            Rect::new(
                0,
                0,
                self.width(),
                self.height(),
            ),
            color,
        );
    }

    fn draw_pixel(
        &mut self,
        point: Point,
        color: Color,
    ) {
        if point.x >= self.width()
            || point.y >= self.height()
        {
            return;
        }

        let pixel_offset =
            point.y * self.info.stride
                + point.x;

        let byte_offset =
            pixel_offset * self.info.bytes_per_pixel;

        if byte_offset + 2 >= self.buffer.len() {
            return;
        }

        match self.info.pixel_format {
            PixelFormat::Rgb => {
                self.buffer[byte_offset] = color.r;
                self.buffer[byte_offset + 1] = color.g;
                self.buffer[byte_offset + 2] = color.b;
            }

            PixelFormat::Bgr => {
                self.buffer[byte_offset] = color.b;
                self.buffer[byte_offset + 1] = color.g;
                self.buffer[byte_offset + 2] = color.r;
            }

            PixelFormat::U8 => {
                self.buffer[byte_offset] = color.r;
            }

            _ => {}
        }
    }

    fn fill_rect(
        &mut self,
        rect: Rect,
        color: Color,
    ) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.draw_pixel(
                    Point::new(x, y),
                    color,
                );
            }
        }
    }

    fn blend_pixel(
        &mut self,
        point: Point,
        color: Color,
        alpha: u8,
    ) {
        if point.x >= self.width()
            || point.y >= self.height()
        {
            return;
        }

        let pixel_offset =
            point.y * self.info.stride
                + point.x;

        let byte_offset =
            pixel_offset * self.info.bytes_per_pixel;

        if byte_offset + 2 >= self.buffer.len() {
            return;
        }

        let alpha = alpha as u16;
        let inverse_alpha = 255 - alpha;

        match self.info.pixel_format {
            PixelFormat::Rgb => {
                let old_red =
                    self.buffer[byte_offset] as u16;

                let old_green =
                    self.buffer[byte_offset + 1] as u16;

                let old_blue =
                    self.buffer[byte_offset + 2] as u16;

                self.buffer[byte_offset] =
                    ((color.r as u16 * alpha
                        + old_red * inverse_alpha)
                        / 255) as u8;

                self.buffer[byte_offset + 1] =
                    ((color.g as u16 * alpha
                        + old_green * inverse_alpha)
                        / 255) as u8;

                self.buffer[byte_offset + 2] =
                    ((color.b as u16 * alpha
                        + old_blue * inverse_alpha)
                        / 255) as u8;
            }

            PixelFormat::Bgr => {
                let old_blue =
                    self.buffer[byte_offset] as u16;

                let old_green =
                    self.buffer[byte_offset + 1] as u16;

                let old_red =
                    self.buffer[byte_offset + 2] as u16;

                self.buffer[byte_offset] =
                    ((color.b as u16 * alpha
                        + old_blue * inverse_alpha)
                        / 255) as u8;

                self.buffer[byte_offset + 1] =
                    ((color.g as u16 * alpha
                        + old_green * inverse_alpha)
                        / 255) as u8;

                self.buffer[byte_offset + 2] =
                    ((color.r as u16 * alpha
                        + old_red * inverse_alpha)
                        / 255) as u8;
            }

            PixelFormat::U8 => {
                let old =
                    self.buffer[byte_offset] as u16;

                self.buffer[byte_offset] =
                    ((color.r as u16 * alpha
                        + old * inverse_alpha)
                        / 255) as u8;
            }

            _ => {}
        }
    }
}