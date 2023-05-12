use super::address::*;
use crate::errno::*;

pub trait FrameAllocator {
    fn frame_alloc(&mut self) -> Option<FrameNumber>;
    fn frame_free(&mut self, frame: FrameNumber) -> Result<(), ErrorCode>;
    fn peek(&self) -> Option<FrameNumber>; // the next available frame
}

pub struct LinearFrameAllocator(Option<FrameNumber>);

impl FrameAllocator for LinearFrameAllocator {
    fn frame_alloc(&mut self) -> Option<FrameNumber> {
        if let Some(ref mut f) = self.0 {
            f.next()
        } else {
            None
        }
    }

    fn frame_free(&mut self, frame: FrameNumber) -> Result<(), ErrorCode> {
        Err(EINVAL)
    }

    fn peek(&self) -> Option<FrameNumber> {
        self.0
    }
}

impl LinearFrameAllocator {
    pub fn new(init: FrameNumber) -> Self {
        Self(Some(init))
    }
}
