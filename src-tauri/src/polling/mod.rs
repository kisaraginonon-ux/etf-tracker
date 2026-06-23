// Polling Manager — 자동 갱신 및 백오프
// REQ-F-05: 1/3/5분 주기, 정규장 중에만 자동 폴링, 백오프 정책
// REQ-F-10: 일시정지 시 자동갱신/Provider복구/알림발송 중단, 수동 새로고침 허용

use std::time::Duration;

use crate::calendar::MarketCalendar;

/// 허용된 폴링 주기 (분)
pub const ALLOWED_INTERVALS: &[u64] = &[1, 3, 5];

pub struct PollingManager {
    interval_secs: u64,
    backoff_secs: u64,
    max_backoff_secs: u64,
    consecutive_failures: u32,
    paused: bool,
    calendar: MarketCalendar,
    /// 마지막 수동 새로고침 시각 (Unix epoch secs)
    last_manual_refresh: Option<u64>,
}

impl PollingManager {
    pub fn new(calendar: MarketCalendar) -> Self {
        Self {
            interval_secs: 60,
            backoff_secs: 60,
            max_backoff_secs: 1800, // 30분
            consecutive_failures: 0,
            paused: false,
            calendar,
            last_manual_refresh: None,
        }
    }

    /// 갱신 주기 설정 (1/3/5분만 허용)
    /// 잘못된 값 시 false 반환
    pub fn set_interval(&mut self, minutes: u64) -> bool {
        if !ALLOWED_INTERVALS.contains(&minutes) {
            tracing::warn!("Invalid polling interval: {} min (allowed: {:?})", minutes, ALLOWED_INTERVALS);
            return false;
        }
        self.interval_secs = minutes * 60;
        // 백오프가 기본값일 때만 interval에 맞춤
        if self.consecutive_failures == 0 {
            self.backoff_secs = self.interval_secs;
        }
        tracing::info!("Polling interval set to {} min", minutes);
        true
    }

    /// 현재 설정된 폴링 주기 (분)
    pub fn interval_minutes(&self) -> u64 {
        self.interval_secs / 60
    }

    /// 일시정지 (REQ-F-10)
    pub fn pause(&mut self) {
        self.paused = true;
        tracing::info!("Polling paused");
    }

    /// 재개 (REQ-F-10)
    pub fn resume(&mut self) {
        self.paused = false;
        self.consecutive_failures = 0;
        self.backoff_secs = self.interval_secs;
        tracing::info!("Polling resumed");
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// 자동 폴링 가능 여부
    /// - 일시정지 아님
    /// - 정규장 중
    pub fn can_auto_poll(&self) -> bool {
        !self.paused && self.calendar.is_regular()
    }

    /// 수동 새로고침 가능 여부
    /// - 일시정지 중에도 수동은 허용 (REQ-F-05, REQ-F-10)
    pub fn can_manual_refresh(&self) -> bool {
        true
    }

    /// 수동 새로고침 기록
    pub fn record_manual_refresh(&mut self) {
        self.last_manual_refresh = Some(
            chrono::Utc::now().timestamp() as u64
        );
    }

    /// 백오프 적용 — 실패 시 기하급수적 증가 (REQ-F-05)
    /// 1min → 2min → 4min → 8min → ... → max 30min
    pub fn on_failure(&mut self) {
        self.consecutive_failures += 1;
        self.backoff_secs = (self.backoff_secs * 2).min(self.max_backoff_secs);
        tracing::warn!(
            "Polling failure #{} — backoff: {}s",
            self.consecutive_failures,
            self.backoff_secs
        );
    }

    /// 성공 시 백오프 리셋
    pub fn on_success(&mut self) {
        if self.consecutive_failures > 0 {
            tracing::info!("Polling recovered after {} failures", self.consecutive_failures);
        }
        self.consecutive_failures = 0;
        self.backoff_secs = self.interval_secs;
    }

    /// 현재 대기 시간 (백오프 적용)
    pub fn current_delay(&self) -> Duration {
        if self.consecutive_failures > 0 {
            Duration::from_secs(self.backoff_secs)
        } else {
            Duration::from_secs(self.interval_secs)
        }
    }

    /// 백오프 단계 (0 = 정상, 1+ = 백오프 중)
    pub fn backoff_level(&self) -> u32 {
        self.consecutive_failures
    }

    /// 현재 상태 요약 (UI 표시용)
    pub fn status_summary(&self) -> PollingStatus {
        PollingStatus {
            paused: self.paused,
            interval_minutes: self.interval_minutes(),
            backoff_level: self.consecutive_failures,
            current_delay_secs: self.current_delay().as_secs(),
            can_auto_poll: self.can_auto_poll(),
            market_state: self.calendar.current_state(),
        }
    }
}

/// 폴링 상태 (UI 전달용)
#[derive(Debug, Clone, serde::Serialize)]
pub struct PollingStatus {
    pub paused: bool,
    pub interval_minutes: u64,
    pub backoff_level: u32,
    pub current_delay_secs: u64,
    pub can_auto_poll: bool,
    pub market_state: crate::calendar::MarketState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::MarketCalendar;

    fn make_manager() -> PollingManager {
        PollingManager::new(MarketCalendar::new())
    }

    #[test]
    fn test_default_interval() {
        let m = make_manager();
        assert_eq!(m.interval_minutes(), 1);
    }

    #[test]
    fn test_set_valid_interval() {
        let mut m = make_manager();
        assert!(m.set_interval(3));
        assert_eq!(m.interval_minutes(), 3);
        assert!(m.set_interval(5));
        assert_eq!(m.interval_minutes(), 5);
    }

    #[test]
    fn test_set_invalid_interval() {
        let mut m = make_manager();
        assert!(!m.set_interval(2));
        assert!(!m.set_interval(10));
        assert_eq!(m.interval_minutes(), 1); // unchanged
    }

    #[test]
    fn test_pause_resume() {
        let mut m = make_manager();
        assert!(!m.is_paused());
        m.pause();
        assert!(m.is_paused());
        assert!(!m.can_auto_poll());
        m.resume();
        assert!(!m.is_paused());
    }

    #[test]
    fn test_backoff_progression() {
        let mut m = make_manager();
        // 60s → 120s → 240s → 480s → 960s → 1800s (capped)
        m.on_failure();
        assert_eq!(m.backoff_level(), 1);
        assert_eq!(m.current_delay().as_secs(), 120);

        m.on_failure();
        assert_eq!(m.current_delay().as_secs(), 240);

        m.on_failure();
        assert_eq!(m.current_delay().as_secs(), 480);

        m.on_failure();
        assert_eq!(m.current_delay().as_secs(), 960);

        m.on_failure();
        assert_eq!(m.current_delay().as_secs(), 1800); // capped at 30min

        // recovery
        m.on_success();
        assert_eq!(m.backoff_level(), 0);
        assert_eq!(m.current_delay().as_secs(), 60);
    }

    #[test]
    fn test_manual_refresh_always_allowed() {
        let mut m = make_manager();
        m.pause();
        assert!(m.can_manual_refresh()); // even when paused
        m.record_manual_refresh();
        assert!(m.last_manual_refresh.is_some());
    }

    #[test]
    fn test_status_summary() {
        let m = make_manager();
        let s = m.status_summary();
        assert_eq!(s.interval_minutes, 1);
        assert!(!s.paused);
        assert_eq!(s.backoff_level, 0);
    }
}