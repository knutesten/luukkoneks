use bitfield::bitfield;

bitfield! {
    pub struct SpriteAttributes(u8);
    impl Debug;
    pub obj_bg_priority, _: 7;
    pub y_flip, _ : 6;
    pub x_flip, _ : 5;
    pub palette_num, _ : 4;
    pub tile_vram_bank, _ : 3;
    pub palette_num_cgb, _ : 2, 0;
}

impl From<u8> for SpriteAttributes {
    fn from(value: u8) -> Self {
        SpriteAttributes(value)
    }
}

bitfield! {
    pub struct OAMEntry([u8]);
    impl Debug;
    u8;
    pub y_pos, _ : 7, 0;
    pub x_pos, _ : 15, 8;
    pub tile_num, _ : 23, 16;
    pub u8, into SpriteAttributes, attributes, _: 31, 24;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set_read() {
        let memory: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut part: [u8; 4] = Default::default();
        part.copy_from_slice(&memory[0..4]);

        let a = OAMEntry(part);
        let y = a.y_pos();
        assert_eq!(y, 1)
    }
}