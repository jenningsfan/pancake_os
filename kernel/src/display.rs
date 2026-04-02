use core::fmt::{self, Write};
use bootloader_api::info::{FrameBuffer, FrameBufferInfo};
use uart_16550::SerialPort;
use crate::psf::Psf1;


pub struct Display<'a> {
    fb: Option<&'a mut FrameBuffer>,
    pub width: usize,
    pub height: usize,
}

impl<'a> Display<'a> {
    pub fn new(fb: Option<&'a mut FrameBuffer>) -> Self {
        let mut width = 0;
        let mut height = 0;

        if let Some(fb) = &fb {
            width = fb.info().width;
            height = fb.info().height;
        }

        Self {
            fb,
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        if let Some(fb) = &mut self.fb {
            fb.buffer_mut().fill(0);
        }
    }

    pub fn draw_glyph(&mut self, x: usize, y: usize, w: usize, h: usize, glyph: &[u8]) {
        if let Some(fb) = &mut self.fb {
            let width = fb.info().width;

            for r in 0..h {
                for c in 0..w {
                    let p_pos = x + c + (y + r) * width;
                    if (glyph[r] >> (w - 1 - c)) & 0x1 == 0x1 {
                        fb.buffer_mut()[p_pos * 3] = 0xFF;
                        fb.buffer_mut()[p_pos * 3 + 1] = 0xFF;
                        fb.buffer_mut()[p_pos * 3 + 2] = 0xFF;
                    }
                }
            }
        }
    }

    pub fn scroll_lines_up(&mut self, num_lines: usize) {
        if let Some(fb) = &mut self.fb {
            let src = num_lines * self.width * 3;
            let buf = fb.buffer_mut();
            let bottom = buf.len() - src - 1;
            buf.copy_within(src.., 0);
            buf[bottom..].fill(0x00);
        }
    }
}

pub struct TTY<'a> {
    x: usize,
    y: usize,
    display: &'a mut Display<'a>,
    serial: &'a mut uart_16550::SerialPort,
    psf1: Psf1<'a>,
}

impl<'a> TTY<'a> {
    pub fn new(display: &'a mut Display<'a>, serial: &'a mut uart_16550::SerialPort) -> Option<Self> {
        Some(Self {
            x: 0,
            y: 0,
            display,
            serial,
            psf1: Psf1::new(crate::psf::FONT)?,
        })
    }

    fn write_string(&mut self, s: &str) {
        self.serial.write_str(s);
        
        for c in s.chars() {
            if c == '\n' {
                self.scroll_line();
                continue;
            }
            let glyph = self.psf1.glyph(c);
            
            self.display.draw_glyph(self.x * self.psf1.width, self.y * self.psf1.height, self.psf1.width, self.psf1.height, glyph);
            self.x += 1;

            if self.x >= self.display.width / self.psf1.width {
                self.scroll_line();
            }
        }
    }

    fn scroll_line(&mut self) {
        self.x = 0;
        self.y += 1;

        if self.y >= self.display.height / self.psf1.height {
            self.display.scroll_lines_up(self.psf1.height);
            self.y -= 1;
        }
    }
}

impl<'a> fmt::Write for TTY<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}