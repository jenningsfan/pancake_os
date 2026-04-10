pub static FONT: &[u8] = include_bytes!("../../fonts/spleen/spleen-8x16.psfu");

use bitflags::bitflags;

const PSF1_MAGIC0: u8 = 0x36;
const PSF1_MAGIC1: u8 = 0x04;

bitflags! {
    struct Psf1Mode: u8 {
        const Chars512 = 0x01;
        const ModeHasTab = 0x02;
        const HasEq = 0x04;
    }
}

pub struct Psf1<'a> {
    magic: [u8; 2],
    _mode: Psf1Mode,
    pub width: usize,
    pub height: usize,
    glyphs: &'a[u8],
}

impl<'a> Psf1<'a> {
    pub fn new(font: &'a[u8]) -> Option<Self> {
        let psf1 = Self {
            magic: [font[0], font[1]],
            _mode: Psf1Mode::from_bits(font[2])?,
            width: 8,
            height: font[3] as usize,
            glyphs: &font[4..],
        };

        if psf1.magic != [PSF1_MAGIC0, PSF1_MAGIC1] {
            None
        }
        else {
            Some(psf1)
        }
    }

    pub fn glyph(&self, c: char) -> &[u8] {
        let c = c as usize;
        &self.glyphs[(c * self.height)..(c + 1) * self.height]
    }
}