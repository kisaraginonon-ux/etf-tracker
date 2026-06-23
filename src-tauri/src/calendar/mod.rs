// Calendar — Market Calendar Engine
// REQ-F-05: 정규장 시간 기반 폴링 제어
// REQ-F-15: 캘린더 누락 시 기본 정규장 시간 Fallback 모드

pub mod schedule;
pub mod holidays;

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// 한국 시간(KST) = UTC+9
const KST_OFFSET: chrono::FixedOffset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();

/// 장 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketState {
    PreOpen,  // 장전 (00:00~09:00)
    Regular,  // 정규장 (09:00~15:30)
    Closed,   // 장 종료 (15:30~24:00)
    Holiday,  // 휴장일 (주말/공휴일)
    Unknown,  // 캘린더 누락 → Fallback 모드
}

impl std::fmt::Display for MarketState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketState::PreOpen => write!(f, "장전"),
            MarketState::Regular => write!(f, "정규장"),
            MarketState::Closed => write!(f, "장마감"),
            MarketState::Holiday => write!(f, "시장 미운영"),
            MarketState::Unknown => write!(f, "알 수 없음"),
        }
    }
}

pub struct MarketCalendar {
    holidays: Vec<NaiveDate>,
    schedule: schedule::MarketSchedule,
    calendar_loaded: bool,
}

impl Default for MarketCalendar {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketCalendar {
    pub fn new() -> Self {
        Self {
            holidays: Vec::new(),
            schedule: schedule::MarketSchedule::default(),
            calendar_loaded: false,
        }
    }

    /// 캘린더 JSON에서 휴장일 로드
    /// 로드 실패 시 Fallback 모드 (calendar_loaded = false)
    pub fn load_holidays(&mut self, json_path: &str) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(json_path)?;
        let cal: holidays::CalendarJson = serde_json::from_str(&content)?;
        self.holidays = cal
            .holidays
            .iter()
            .filter_map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .collect();
        self.calendar_loaded = true;
        tracing::info!("Loaded {} holidays from {}", self.holidays.len(), json_path);
        Ok(())
    }

    /// 캘린더 로드 여부
    pub fn is_loaded(&self) -> bool {
        self.calendar_loaded
    }

    /// 현재 KST 시간 가져오기
    fn now_kst() -> chrono::DateTime<chrono::FixedOffset> {
        Utc::now().with_timezone(&KST_OFFSET)
    }

    /// 특정 KST 시간 기준 장 상태 판별 (테스트용)
    pub fn state_at(&self, kst: chrono::DateTime<chrono::FixedOffset>) -> MarketState {
        let today = kst.date_naive();

        // 주말 체크
        let weekday = today.weekday();
        if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
            return MarketState::Holiday;
        }

        // 휴장일 체크 (캘린더 로드된 경우만)
        if self.calendar_loaded && self.holidays.contains(&today) {
            return MarketState::Holiday;
        }

        // 시간 기반 상태 판별
        let time = kst.time();
        let reg_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let reg_end = NaiveTime::from_hms_opt(15, 30, 0).unwrap();

        if time >= reg_start && time <= reg_end {
            MarketState::Regular
        } else if time < reg_start {
            MarketState::PreOpen
        } else {
            MarketState::Closed
        }
    }

    /// 현재 장 상태 판별 (KST 기준)
    pub fn current_state(&self) -> MarketState {
        self.state_at(Self::now_kst())
    }

    /// 정규장 여부
    pub fn is_regular(&self) -> bool {
        self.current_state() == MarketState::Regular
    }

    /// 정규장 경과 분 (09:00 기준, 최소 0)
    pub fn elapsed_minutes(&self) -> u32 {
        let now = Self::now_kst();
        let reg_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let elapsed = (now.time() - reg_start).num_minutes();
        if elapsed < 0 { 0 } else { elapsed.min(390) as u32 }
    }

    /// 전체 정규장 분 (09:00~15:30 = 390분)
    pub fn total_regular_minutes(&self) -> u32 {
        390 // 6.5시간 × 60
    }

    /// 정규장 진행률 (0.0 ~ 1.0)
    pub fn regular_progress(&self) -> f64 {
        let total = self.total_regular_minutes() as f64;
        let elapsed = self.elapsed_minutes() as f64;
        (elapsed / total).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kst_time(h: u32, m: u32) -> chrono::DateTime<chrono::FixedOffset> {
        let date = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap(); // 화요일
        let dt = NaiveDateTime::new(date, NaiveTime::from_hms_opt(h, m, 0).unwrap());
        KST_OFFSET.from_local_datetime(&dt).unwrap()
    }

    #[test]
    fn test_regular_hours() {
        let cal = MarketCalendar::new();
        assert_eq!(cal.state_at(kst_time(10, 0)), MarketState::Regular);
        assert_eq!(cal.state_at(kst_time(9, 0)), MarketState::Regular);
        assert_eq!(cal.state_at(kst_time(15, 30)), MarketState::Regular);
    }

    #[test]
    fn test_pre_open() {
        let cal = MarketCalendar::new();
        assert_eq!(cal.state_at(kst_time(8, 59)), MarketState::PreOpen);
        assert_eq!(cal.state_at(kst_time(0, 0)), MarketState::PreOpen);
    }

    #[test]
    fn test_closed() {
        let cal = MarketCalendar::new();
        assert_eq!(cal.state_at(kst_time(15, 31)), MarketState::Closed);
        assert_eq!(cal.state_at(kst_time(23, 59)), MarketState::Closed);
    }

    #[test]
    fn test_weekend() {
        let cal = MarketCalendar::new();
        let sat = NaiveDate::from_ymd_opt(2026, 6, 27).unwrap(); // 토요일
        let dt = NaiveDateTime::new(sat, NaiveTime::from_hms_opt(10, 0, 0).unwrap());
        let kst = KST_OFFSET.from_local_datetime(&dt).unwrap();
        assert_eq!(cal.state_at(kst), MarketState::Holiday);
    }

    #[test]
    fn test_holiday_loaded() {
        let mut cal = MarketCalendar::new();
        cal.holidays.push(NaiveDate::from_ymd_opt(2026, 6, 23).unwrap());
        cal.calendar_loaded = true;
        // 6/23이 휴장일로 등록된 경우
        assert_eq!(cal.state_at(kst_time(10, 0)), MarketState::Holiday);
    }

    #[test]
    fn test_holiday_not_loaded_fallback() {
        // 캘린더 미로드 시 휴장일 체크 안함 (Fallback)
        let cal = MarketCalendar::new();
        // 6/23이 실제 휴장일이더라도 calendar_loaded=false면 정규장 시간이면 Regular
        assert_eq!(cal.state_at(kst_time(10, 0)), MarketState::Regular);
    }

    #[test]
    fn test_elapsed_minutes() {
        let cal = MarketCalendar::new();
        // elapsed는 현재 시간 기준이므로 0~390 범위
        let elapsed = cal.elapsed_minutes();
        assert!(elapsed <= 390);
    }
}