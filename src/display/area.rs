#[derive(Debug, PartialEq)]
pub struct Area {
    pub left: u16,
    pub top: u16,
    pub width: u16,
    pub height: u16,
}

impl Area {
    pub fn new(
        left: u16,
        top: u16,
        width: u16,
        height: u16,
    ) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }
}
