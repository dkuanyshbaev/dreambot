//////////////////////////////////////////
// Tzolkin core lib
//////////////////////////////////////////
use crate::tables::*;

pub fn kin(day: u32, month: u32, year: i32) -> u32 {
    let year_index = year as f32 - ((year as f32 / 52_f32).floor() * 52_f32);
    let mut kin = day + MONTH_TABLE[month as usize - 1] + YEAR_TABLE[year_index as usize];
    if kin > 260 {
        kin -= 260
    }
    kin
}

pub fn archetype(kin: u32) -> (u32, u32) {
    ARCHETYPE_TABLE[(kin - 1) as usize]
}
