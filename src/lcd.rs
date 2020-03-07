use bitfield::bitfield;

mod frame;
use frame::Frame;
mod oam;
use oam::OAMEntry;

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

    pub fn read(&self, addr: u16) -> u8 {
        return match addr {
            0xFF40 => self.lcdc.0,
            0xFF41 => self.stat.0,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.lcdc_y,
            0xFF34 => self.ly_compare,
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            0xFF47 => self.background_palette,
            0xFF48 => self.object_palette_0,
            0xFF49 => self.object_palette_1,
            0xFF46 => unimplemented!(),
            _ => panic!("Unhandled write to LCD register {}", addr)
        };
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


struct LCD {
    registers: LCDRegisters,
    vram_tile_data: [u8; 0x1800],
    background_map1: [u8; 0x400],
    background_map2: [u8; 0x400],
    sprite_attribute_table: [u8; 0xA0],
    cur_y: u8,
    ticks_since_work: usize,
    lcd_state: LCDMode,
    frame: Frame,

}

#[derive(Clone,Copy)]
enum LCDMode {
    HBLANK,
    VBLANK,
    MODE2,
    MODE3,
}

impl LCD {
    fn new() -> Self {
        let mut lcd = LCD {
            registers: LCDRegisters::new(),
            vram_tile_data: [0; 0x1800],
            sprite_attribute_table: [0; 0xA0],
            background_map1: [0; 0x400],
            background_map2: [0; 0x400],
            cur_y: 0,
            ticks_since_work: 0,
            lcd_state: LCDMode::MODE2,
            frame: Frame::new(),
        };
        lcd.set_mode(LCDMode::MODE2);
        lcd
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40..=0xFF46 => self.registers.read(addr),
            0x8000..=0x97FF => self.vram_tile_data[addr as usize - 0x8000],
            0x9800..=0x9BFF => self.background_map1[addr as usize - 0x9800],
            0x9C00..=0x9FFF => self.background_map2[addr as usize - 0x9C00],
            _ => panic!("Unhandled read in LCD, {:06x}", addr)
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF40..=0xFF46 => self.registers.write(addr, value),
            0x8000..=0x97FF => self.vram_tile_data[addr as usize - 0x8000] = value,
            0x9800..=0x9BFF => self.background_map1[addr as usize - 0x9800] = value,
            0x9C00..=0x9FFF => self.background_map2[addr as usize - 0x9C00] = value,
            _ => panic!("Unhandled write in LCD, {:06x}", addr)
        }
    }

    fn find_bg_window_tile(&self, tile_map_addr: u16, tile_num: u16) -> &[u8] {
        let bg_window_tile_data_select = self.registers.lcdc.bg_window_tile_data_select();
        let tile_data_addr: u16 = if bg_window_tile_data_select { 0x8000 } else { 0x8800 };

        let tile_idx = self.read(tile_map_addr + tile_num);
        let start_tile_data = tile_data_addr + (tile_idx as u16 * 16);
        let start_tile_data = (start_tile_data - 0x8000) as usize;
        let end_tile_data = start_tile_data + 16;
        return &self.vram_tile_data[start_tile_data..end_tile_data];
    }

    fn find_bg_tile(&self, tile_num: u16) -> &[u8] {
        let bg_tile_map_select = self.registers.lcdc.bg_tile_map_display_select();
        let tile_map_addr: u16 = if bg_tile_map_select { 0x9C00 } else { 0x9800 };
        self.find_bg_window_tile(tile_map_addr, tile_num)
    }

    fn find_window_tile(&self, tile_num: u16) -> &[u8] {
        let window_tile_map_select = self.registers.lcdc.window_tile_map_display();
        let tile_map_addr: u16 = if window_tile_map_select { 0x9C00 } else { 0x9800 };
        self.find_bg_window_tile(tile_map_addr, tile_num)
    }

    fn find_tile_index(y: u8, x: u8) -> u16 {
        let y = y as u16;
        let x = x as u16;
        return (y / 8) * 32 + (x / 8);
    }

    fn find_pixel_in_tile(tile: &[u8], y: u8, x: u8) -> u8 {
        assert_eq!(tile.len(), 16);
        /* Pixel format, 2 bits per pixel
         * 4 pixels in 1 byte
         * A line of 8 pixels is then 16 bits (2 bytes)
         */
        let y = y % 8;
        let x = x % 8;
        let byte1 = tile[(y * 2) as usize];
        let byte2 = tile[((y * 2) + 1) as usize];
        let shift = 7 - (x % 8);
        let lsb = (byte1 >> shift) & 0x01;
        let msb = (byte2 >> shift) & 0x01;
        return msb << 1 | lsb;
    }

    fn bg_pixel_to_color(pixel: u8) -> u8 {
        // TODO: lookup
        match pixel {
            0 => 0xFF,
            1 => 0x80,
            2 => 0x40,
            3 => 0x00,
            _ => panic!("Pixel value out of bounds")
        }
    }

    fn window_pixel_to_color(pixel: u8) -> u8 {
        // TODO: lookup
        match pixel {
            0 => 0xFF,
            1 => 0x80,
            2 => 0x40,
            3 => 0x00,
            _ => panic!("Pixel value out of bounds")
        }
    }

    fn find_tiles_to_draw(&self, cur_y: u8) -> Vec<OAMEntry<[u8; 4]>> {
        // TODO: In a real GB only 10 sprites per line are drawn
        let mut tiles: Vec<OAMEntry<[u8; 4]>> = Vec::new();
        let sprite_height = if self.registers.lcdc.obj_size() { 16 } else { 8 };
        for i in 0..40 {
            let start_idx = i * 4;
            let end_idx = start_idx + 4;
            let mut data: [u8; 4] = Default::default();
            data.copy_from_slice(&self.sprite_attribute_table[start_idx..end_idx]);
            let entry = OAMEntry(data);
            if entry.y_pos() <= cur_y && (entry.y_pos() + sprite_height) >= cur_y {
                tiles.push(entry)
            }
        }
        return tiles;
    }

    fn draw_line(&mut self) {
        let cur_y = self.cur_y;
        let bg_y = self.registers.scroll_y + self.cur_y;
        let window_x = self.registers.scroll_x;

        let bg_window_display = self.registers.lcdc.bg_window_display_priority();
        let window_display_enable = self.registers.lcdc.window_display_enable();

        // Draw background
        for i in 0..160 {
            if bg_window_display {
                let cur_x = window_x + i;
                let tile_index = LCD::find_tile_index(bg_y, cur_x);
                let tile = self.find_bg_tile(tile_index);
                let pixel = LCD::find_pixel_in_tile(tile, bg_y, cur_x);
                self.frame[cur_y * 160 + i] = LCD::bg_pixel_to_color(pixel);
            } else {
                self.frame[cur_y * 160 + i] = 0xFF;
            }
        }

        // Draw window
        let window_y = self.registers.read(0xFF4A);
        let window_x = self.registers.read(0xFF4B);
        let window_visible = window_y < 143 && window_y >= 0 &&
            window_x >= 0 && window_x < 166 && window_y <= cur_y;
        if bg_window_display && window_display_enable && window_visible {
            let window_y_idx = cur_y - window_y;
            let window_actual_x = window_x - 7;
            for i in 0..160 {
                if window_actual_x < i { continue; }
                let tile_index = LCD::find_tile_index(window_y_idx, window_actual_x);
                let tile = self.find_window_tile(tile_index);
                let pixel = LCD::find_pixel_in_tile(tile, window_y_idx, window_actual_x);
                self.frame[cur_y * 160 + i] = LCD::window_pixel_to_color(pixel);
            }
        }

        //let tiles_to_draw
        // Draw objects
        if self.registers.lcdc.obj_display_enable() {
            let tiles = &self.find_tiles_to_draw(self.cur_y);
            for i in 0..160 {

            }
        }
    }

    fn set_mode(&mut self, lcd_mode: LCDMode) {
        self.lcd_state = lcd_mode;
        let stat_value = match lcd_mode {
            LCDMode::HBLANK => 0 as u8,
            LCDMode::VBLANK => 1 as u8,
            LCDMode::MODE2 => 2 as u8,
            LCDMode::MODE3 => 3 as u8,
        };

        self.registers.stat = STAT(self.registers.stat.0 & !(0x3 as u8) | stat_value);
    }

    fn handle_mode2(&mut self) -> bool {
        if self.ticks_since_work > 80 {
            self.ticks_since_work -= 80;
            self.set_mode(LCDMode::MODE3);
            return true;
        }

        return false;
    }

    fn handle_mode3(&mut self) -> bool {
        if self.ticks_since_work > 172 {
            self.ticks_since_work -= 172;
            self.set_mode(LCDMode::HBLANK);
            self.draw_line();
            return true;
        }

        return false;
    }

    fn handle_hblank(&mut self) -> bool {
        if self.ticks_since_work > 203 {
            self.ticks_since_work -= 203;
            self.cur_y += 1;

            if self.cur_y >= 144 {
                self.cur_y = 0;
                self.set_mode(LCDMode::VBLANK);
            } else {
                self.set_mode(LCDMode::MODE2);
            }

            return true;
        }

        return false;
    }

    fn handle_vblank(&mut self) -> bool {
        if self.ticks_since_work > 4560 {
            self.ticks_since_work -= 4560;
            return true;
        }

        return false;
    }

    fn tick(&mut self, ticks: usize) {
        self.ticks_since_work += ticks;
        loop {
            let cont = match self.lcd_state {
                LCDMode::MODE2 => self.handle_mode2(),
                LCDMode::MODE3 => self.handle_mode3(),
                LCDMode::HBLANK => self.handle_hblank(),
                LCDMode::VBLANK => self.handle_vblank(),
            };
            if !cont {
                return;
            }
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

    #[test]
    fn test_box() {
        let mut lcd = Box::new(LCD::new());
    }
}