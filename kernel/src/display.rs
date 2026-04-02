use bootloader_api::info::FrameBuffer;

pub struct Display<'a> {
    fb: Option<&'a mut FrameBuffer>
}

impl<'a> Display<'a> {
    pub fn new(fb: Option<&'a mut FrameBuffer>) -> Self {
        Self {
            fb
        }
    }

    pub fn clear(&mut self) {
        if let Some(fb) = &mut self.fb {
            fb.buffer_mut().fill(0);
        }
    }
}