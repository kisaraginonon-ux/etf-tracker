// Schedule — 정규장 시간 정의

pub struct MarketSchedule {
    pub regular_start: (u32, u32), // (시, 분) = 09:00
    pub regular_end: (u32, u32),   // (시, 분) = 15:30
}

impl Default for MarketSchedule {
    fn default() -> Self {
        Self {
            regular_start: (9, 0),
            regular_end: (15, 30),
        }
    }
}