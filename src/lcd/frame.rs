use std::ops::{Index, IndexMut};

pub struct Frame {
    data: [u8; 160 * 144]
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            data: [0; 160 * 144]
        }
    }
}

impl Index<u8> for Frame {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        return &self.data[index as usize];
    }
}

impl IndexMut<u8> for Frame {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        return &mut self.data[index as usize];
    }
}