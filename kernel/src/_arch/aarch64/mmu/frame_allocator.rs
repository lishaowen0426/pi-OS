use super::address::*;
use crate::errno::*;

pub trait FrameAllocator {
    fn frame_alloc(&mut self) -> Option<FrameNumber>;
    fn frame_free(&mut self, frame: FrameNumber) -> Result<(), ErrorCode>;
}

pub struct LinearFrameAllocator(FrameNumber);

impl FrameAllocator for LinearFrameAllocator {
    fn frame_alloc(&mut self) -> Option<FrameNumber> {
        self.0.next()
    }

    fn frame_free(&mut self, frame: FrameNumber) -> Result<(), ErrorCode> {
        Err(EINVAL)
    }
}

impl LinearFrameAllocator {
    pub fn new(init: FrameNumber) -> Self {
        Self(init)
    }
}
