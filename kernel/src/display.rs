use core::fmt;
use bootloader_api::info::{FrameBuffer, FrameBufferInfo};
use crate::psf::Psf1;


pub struct Display<'a> {
    fb: Option<&'a mut FrameBuffer>,
}

impl<'a> Display<'a> {
    pub fn new(fb: Option<&'a mut FrameBuffer>) -> Self {
        Self {
            fb,
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
}

pub struct TTY<'a> {
    x: usize,
    y: usize,
    display: &'a mut Display<'a>,
    psf1: Option<Psf1<'a>>,
}

impl<'a> TTY<'a> {
    pub fn new(display: &'a mut Display<'a>) -> Self {
        Self {
            x: 0,
            y: 0,
            display,
            psf1: Psf1::new(crate::psf::FONT),
        }
    }

    fn write_string(&mut self, s: &str) {
        if let Some(psf) = &self.psf1 {
            for c in s.chars() {
                if c == '\n' {
                    self.x = 0;
                    self.y += 1;
                    continue;
                }
                
                let glyph = psf.glyph(c);
                self.display.draw_glyph(self.x * psf.width, self.y * psf.height, psf.width, psf.height, glyph);
                self.x += 1;
            }
        }
    }
}

impl<'a> fmt::Write for TTY<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}