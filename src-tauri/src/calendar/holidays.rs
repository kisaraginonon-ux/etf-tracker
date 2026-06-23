// Holidays — 휴장일 로드 및 관리
// market_calendar.json 구조

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CalendarJson {
    pub year: u32,
    pub holidays: Vec<String>, // "YYYY-MM-DD" 형식
}