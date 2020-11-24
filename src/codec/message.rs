use super::Frame;

// -- Message

pub struct Message<'a> {
    pub(crate) frame: Frame<'a>,
    pub(crate) is_last: bool,
}

impl<'a> Message<'a> {
    #[inline]
    pub fn is_last(&self) -> bool {
        self.is_last
    }

    #[inline]
    pub fn body(&self) -> &'a [u8] {
        let start = if self.frame.bytes[0] == 0x0 || self.frame.bytes[0] == 0x1 {
            2
        } else {
            9
        };
        &self.frame.bytes[start..]
    }
}
