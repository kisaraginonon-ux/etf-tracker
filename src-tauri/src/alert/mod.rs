// Alert Engine — 가격 알림 및 Debounce
// REQ-F-11: 목표가/손절가 돌파 시 1회 토스트
// REQ-F-12: 조건 해소 후 재돌파 시 재발송 (Debounce)
// REQ-F-13: Stale/장애/장종료 시 알림 중단

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::models::{AlertType, DataStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlertState {
    target_triggered: bool,
    stop_triggered: bool,
    last_price: f64,
}

pub struct AlertEngine {
    states: HashMap<String, AlertState>,
}

impl AlertEngine {
    pub fn new() -> Self {
        Self { states: HashMap::new() }
    }

    /// 가격 알림 체크 — 돌파 여부 판별
    /// 반환: 발송해야 할 알림 목록 [(AlertType, threshold, current_price)]
    pub fn check(
        &mut self,
        ticker: &str,
        current_price: f64,
        target_price: Option<f64>,
        stop_loss: Option<f64>,
        data_status: DataStatus,
    ) -> Vec<AlertEvent> {
        // REQ-F-13: Stale/장애/장종료/장전/휴장 시 알림 중단
        if data_status != DataStatus::Live {
            return Vec::new();
        }

        let state = self.states.entry(ticker.to_string()).or_insert(AlertState {
            target_triggered: false,
            stop_triggered: false,
            last_price: current_price,
        });

        let mut events = Vec::new();

        // 목표가 (상한) 돌파 체크 — REQ-F-12
        if let Some(target) = target_price {
            if !state.target_triggered && current_price >= target {
                events.push(AlertEvent {
                    ticker: ticker.to_string(),
                    alert_type: AlertType::Target,
                    threshold: target,
                    current_price,
                });
                state.target_triggered = true;
            }
            // 조건 해소 (가격이 기준선 아래로 복귀)
            if state.target_triggered && current_price < target {
                state.target_triggered = false;
            }
        }

        // 손절가 (하한) 돌파 체크 — REQ-F-12
        if let Some(stop) = stop_loss {
            if !state.stop_triggered && current_price <= stop {
                events.push(AlertEvent {
                    ticker: ticker.to_string(),
                    alert_type: AlertType::StopLoss,
                    threshold: stop,
                    current_price,
                });
                state.stop_triggered = true;
            }
            // 조건 해소 (가격이 기준선 위로 복귀)
            if state.stop_triggered && current_price > stop {
                state.stop_triggered = false;
            }
        }

        state.last_price = current_price;
        events
    }

    /// 사용자 재설정 — triggered 상태 초기화 (REQ-F-12)
    pub fn reset(&mut self, ticker: &str) {
        if let Some(state) = self.states.get_mut(ticker) {
            state.target_triggered = false;
            state.stop_triggered = false;
            tracing::info!("Alert reset for {}", ticker);
        }
    }

    /// 특정 종목의 알림 상태 조회
    pub fn get_state(&self, ticker: &str) -> Option<&AlertState> {
        self.states.get(ticker)
    }

    /// 전체 상태 초기화
    pub fn reset_all(&mut self) {
        self.states.clear();
        tracing::info!("All alert states reset");
    }
}

/// 알림 이벤트 (발송용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub ticker: String,
    pub alert_type: AlertType,
    pub threshold: f64,
    pub current_price: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_target(price: f64, target: f64) -> Vec<AlertEvent> {
        let mut engine = AlertEngine::new();
        engine.check("TEST", price, Some(target), None, DataStatus::Live)
    }

    #[test]
    fn test_target_hit_first_time() {
        let events = check_target(15000.0, 14000.0);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].alert_type, AlertType::Target);
    }

    #[test]
    fn test_no_duplicate_target() {
        let mut engine = AlertEngine::new();
        // 첫 돌파
        let e1 = engine.check("TEST", 15000.0, Some(14000.0), None, DataStatus::Live);
        assert_eq!(e1.len(), 1);
        // 같은 상태에서 다시 체크 — 중복 발송 안 함
        let e2 = engine.check("TEST", 15100.0, Some(14000.0), None, DataStatus::Live);
        assert_eq!(e2.len(), 0);
    }

    #[test]
    fn test_retrigger_after_recovery() {
        let mut engine = AlertEngine::new();
        // 돌파
        let e1 = engine.check("TEST", 15000.0, Some(14000.0), None, DataStatus::Live);
        assert_eq!(e1.len(), 1);
        // 가격 복귀 (조건 해소)
        let e2 = engine.check("TEST", 13000.0, Some(14000.0), None, DataStatus::Live);
        assert_eq!(e2.len(), 0);
        // 재돌파
        let e3 = engine.check("TEST", 14100.0, Some(14000.0), None, DataStatus::Live);
        assert_eq!(e3.len(), 1);
    }

    #[test]
    fn test_stale_no_alert() {
        let mut engine = AlertEngine::new();
        let e = engine.check("TEST", 15000.0, Some(14000.0), None, DataStatus::Stale);
        assert_eq!(e.len(), 0);
    }

    #[test]
    fn test_closed_no_alert() {
        let mut engine = AlertEngine::new();
        let e = engine.check("TEST", 15000.0, Some(14000.0), None, DataStatus::MarketClosed);
        assert_eq!(e.len(), 0);
    }

    #[test]
    fn test_stop_loss_hit() {
        let mut engine = AlertEngine::new();
        let e = engine.check("TEST", 9000.0, None, Some(10000.0), DataStatus::Live);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].alert_type, AlertType::StopLoss);
    }

    #[test]
    fn test_reset() {
        let mut engine = AlertEngine::new();
        engine.check("TEST", 15000.0, Some(14000.0), None, DataStatus::Live);
        engine.reset("TEST");
        let e = engine.check("TEST", 15000.0, Some(14000.0), None, DataStatus::Live);
        assert_eq!(e.len(), 1); // reset 후 재발송
    }

    #[test]
    fn test_both_target_and_stop() {
        let mut engine = AlertEngine::new();
        // target=15000, stop=10000, 현재 16000 → target만
        let e = engine.check("TEST", 16000.0, Some(15000.0), Some(10000.0), DataStatus::Live);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].alert_type, AlertType::Target);
    }
}