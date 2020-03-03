use bitfield::bitfield;

bitfield! {
    pub struct LCDC(u8);
    impl Debug;
    pub enable, _: 7;
    pub window_tile_map_display, _: 6;
    pub window_display_enable, _ : 5;
    pub bg_window_tile_data_select, _ : 4;
    pub bg_tile_map_display_select, _ : 3;
    pub obj_size, _ : 2;
    pub obj_display_enable, _ : 1;
    pub bg_window_display_priority, _ : 0;
}

bitfield! {
    pub struct STAT(u8);
    impl Debug;
    pub ly_coincidence_int, _: 6;
    pub mode_2_oam_int, _: 5;
    pub mode_1_vblank_int, _: 4;
    pub mode_0_hblank_int, _: 3;
    pub coincidence_flag, _: 2;
    pub mode_flag, _: 1, 0;
}



struct LCDRegisters {
    lcdc: LCDC,
    stat: STAT,
    scroll_y: u8,
    scroll_x: u8,
    lcdc_y: u8,
    ly_compare: u8,
    window_y: u8,
    window_x: u8,
    background_palette: u8,
    object_palette_0: u8,
    object_palette_1: u8,
}

impl LCDRegisters {
    fn new() -> LCDRegisters {
        LCDRegisters {
            lcdc: LCDC(0),
            stat: STAT(0),
            scroll_y: 0,
            scroll_x: 0,
            lcdc_y: 0,
            ly_compare: 0,
            window_y: 0,
            window_x: 0,
            background_palette: 0,
            object_palette_0: 0,
            object_palette_1: 0,
        }
    }

    fn set_stat(&mut self, value: u8) {
        const MASK: u8 = 0b1111000;
        let cur_stat = self.stat.0;
        let new_stat = (cur_stat & !MASK) | (value & MASK);
        self.stat = STAT(new_stat)
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF40 => self.lcdc = LCDC(value),
            0xFF41 => self.stat = STAT(value),
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => panic!("Cannot write to LCDC Y coordinate"),
            0xFF34 => self.ly_compare = value,
            0xFF4A => self.window_y = value,
            0xFF4B => self.window_x = value,
            0xFF47 => self.background_palette = value,
            0xFF48 => self.object_palette_0 = value,
            0xFF49 => self.object_palette_1 = value,
            0xFF46 => unimplemented!(),
            _ => panic!("Unhandled write to LCD register {}", addr)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set_read() {
        let mut lcdregs = LCDRegisters::new();
        lcdregs.lcdc = LCDC(0x80);
        println!("Test {:?}", lcdregs.lcdc);
    }
}