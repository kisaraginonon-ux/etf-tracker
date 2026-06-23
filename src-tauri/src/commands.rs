// Commands — Tauri IPC Commands (Frontend ↔ Backend)
// TECH_DESIGN.md §10 참조

use std::sync::Mutex;
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::calendar::{MarketCalendar, MarketState};
use crate::db::Database;
use crate::models::{EtfMasterItem, EtfListItem, Favorite, NormalizedQuote};
use crate::providers::{NaverProvider, YahooProvider, ProviderManager};

/// ETF 마스터 JSON 로드
fn load_etf_master() -> Vec<EtfMasterItem> {
    let paths = [
        "src-tauri/data/etf_master.json",
        "data/etf_master.json",
    ];
    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(parsed) = serde_json::from_str::<EtfMasterList>(&content) {
                return parsed.etf_list;
            }
        }
    }
    Vec::new()
}

/// 캘린더 JSON 경로 탐색
fn calendar_path() -> Option<String> {
    let paths = [
        "src-tauri/data/market_calendar.json",
        "data/market_calendar.json",
    ];
    for path in &paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    None
}

#[derive(Deserialize)]
struct EtfMasterList {
    etf_list: Vec<EtfMasterItem>,
}

/// 현재 장 상태 조회
#[tauri::command]
pub fn get_market_state(calendar: State<'_, Mutex<MarketCalendar>>) -> MarketState {
    let cal = calendar.lock().unwrap();
    cal.current_state()
}

/// 정규장 진행률 (0.0~1.0) — 모멘텀 시그널 계산용
#[tauri::command]
pub fn get_market_progress(calendar: State<'_, Mutex<MarketCalendar>>) -> f64 {
    let cal = calendar.lock().unwrap();
    cal.regular_progress()
}

/// ETF 종목 검색 (마스터 JSON 기반)
#[tauri::command]
pub fn search_etf(query: String) -> Vec<EtfMasterItem> {
    let master = load_etf_master();
    if query.is_empty() {
        return master;
    }
    let query_lower = query.to_lowercase();
    master
        .into_iter()
        .filter(|item| {
            item.name.to_lowercase().contains(&query_lower)
                || item.ticker.contains(&query)
        })
        .collect()
}

/// 즐겨찾기 목록 조회
#[tauri::command]
pub fn get_favorites(db: State<'_, Database>) -> Result<Vec<Favorite>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::models::get_favorites(&conn).map_err(|e| e.to_string())
}

/// 즐겨찾기 추가
#[tauri::command]
pub fn add_favorite(
    db: State<'_, Database>,
    ticker: String,
    name: String,
    market_section: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::models::add_favorite(&conn, &ticker, &name, &market_section)
        .map_err(|e| e.to_string())
}

/// 즐겨찾기 제거
#[tauri::command]
pub fn remove_favorite(db: State<'_, Database>, ticker: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::models::remove_favorite(&conn, &ticker).map_err(|e| e.to_string())
}

// === Virtual Position Commands (REQ-F-07, REQ-F-08) ===

use crate::models::VirtualPosition;

/// 가상 포지션 설정 (매수일, 평균단가, 수량)
#[tauri::command]
pub fn set_position(
    db: State<'_, Database>,
    ticker: String,
    buy_date: Option<String>,
    avg_buy_price: Option<f64>,
    quantity: Option<f64>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::models::set_position(&conn, &ticker, buy_date.as_deref(), avg_buy_price, quantity)
        .map_err(|e| e.to_string())
}

/// 가상 포지션 목록 조회
#[tauri::command]
pub fn get_positions(db: State<'_, Database>) -> Result<Vec<VirtualPosition>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::models::get_positions(&conn).map_err(|e| e.to_string())
}

/// 가상 포지션 삭제
#[tauri::command]
pub fn remove_position(db: State<'_, Database>, ticker: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM virtual_positions WHERE ticker = ?1", rusqlite::params![ticker])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// === Alert Commands (REQ-F-11, REQ-F-12, REQ-F-13) ===

use crate::models::{AlertConfig, AlertType};

/// 알림 설정 (목표가/손절가)
#[tauri::command]
pub fn set_alert(
    db: State<'_, Database>,
    ticker: String,
    alert_type: String,
    threshold: f64,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::models::set_alert(&conn, &ticker, &alert_type, threshold)
        .map_err(|e| e.to_string())
}

/// 알림 목록 조회
#[tauri::command]
pub fn get_alerts(db: State<'_, Database>) -> Result<Vec<AlertConfig>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::models::get_alerts(&conn).map_err(|e| e.to_string())
}

/// 알림 재설정 (triggered 상태 초기화 — REQ-F-12)
#[tauri::command]
pub fn reset_alert(
    db: State<'_, Database>,
    ticker: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE alerts SET triggered_at = NULL, triggered_price = NULL WHERE ticker = ?1",
        rusqlite::params![ticker],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// 알림 비활성화
#[tauri::command]
pub fn remove_alert(
    db: State<'_, Database>,
    ticker: String,
    alert_type: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM alerts WHERE ticker = ?1 AND alert_type = ?2",
        rusqlite::params![ticker, alert_type],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// === Polling Commands (REQ-F-05, REQ-F-10) ===

use crate::polling::{PollingManager, PollingStatus};

/// 폴링 주기 변경 (1/3/5분)
#[tauri::command]
pub fn set_polling_interval(
    polling: State<'_, Mutex<PollingManager>>,
    minutes: u64,
) -> Result<bool, String> {
    let mut mgr = polling.lock().map_err(|e| e.to_string())?;
    Ok(mgr.set_interval(minutes))
}

/// 일시정지
#[tauri::command]
pub fn pause_polling(polling: State<'_, Mutex<PollingManager>>) -> Result<(), String> {
    let mut mgr = polling.lock().map_err(|e| e.to_string())?;
    mgr.pause();
    Ok(())
}

/// 재개
#[tauri::command]
pub fn resume_polling(polling: State<'_, Mutex<PollingManager>>) -> Result<(), String> {
    let mut mgr = polling.lock().map_err(|e| e.to_string())?;
    mgr.resume();
    Ok(())
}

/// 폴링 상태 조회
#[tauri::command]
pub fn get_polling_status(polling: State<'_, Mutex<PollingManager>>) -> Result<PollingStatus, String> {
    let mgr = polling.lock().map_err(|e| e.to_string())?;
    Ok(mgr.status_summary())
}

/// 수동 새로고침 (일시정지 중에도 허용)
#[tauri::command]
pub fn manual_refresh(
    polling: State<'_, Mutex<PollingManager>>,
) -> Result<String, String> {
    let mut mgr = polling.lock().map_err(|e| e.to_string())?;
    if !mgr.can_manual_refresh() {
        return Err("Manual refresh not available".to_string());
    }
    mgr.record_manual_refresh();
    // TODO: 실제 데이터 fetch는 Phase 5에서 Provider 연동
    Ok("refreshed".to_string())
}

// === CSV Export (REQ-F-16) ===

use chrono::FixedOffset;

/// Provider 상태 정보 (프론트엔드 Fallback 배너용)
#[derive(Debug, Clone, Serialize)]
pub struct ProviderStatus {
    pub active_provider: String,
    pub is_using_fallback: bool,
    pub primary_failures: u32,
}

/// Provider 전환 상태 조회 (REQ-F-19)
#[tauri::command]
pub async fn get_provider_status(
    provider_mgr: State<'_, tokio::sync::Mutex<ProviderManager>>,
) -> Result<ProviderStatus, String> {
    let mgr = provider_mgr.lock().await;
    Ok(ProviderStatus {
        active_provider: mgr.active_provider().to_string(),
        is_using_fallback: mgr.is_using_fallback(),
        primary_failures: mgr.primary_failures(),
    })
}

// === Settings (REQ-NF-05) ===

/// 설정 값 조회
#[tauri::command]
pub fn get_setting(db: State<'_, Database>, key: String) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let value: Option<String> = conn
        .query_row("SELECT value FROM settings WHERE key = ?1", rusqlite::params![key], |row| row.get(0))
        .ok();
    Ok(value.unwrap_or_default())
}

/// 설정 값 저장
#[tauri::command]
pub fn set_setting(db: State<'_, Database>, key: String, value: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        rusqlite::params![key, value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// CSV 내보내기 (REQ-F-16)
/// - UTF-8 BOM 인코딩 (Excel 호환)
/// - KST 시간 표기
/// - 파일명: ETF_Export_YYYYMMDD_HHMM.csv
/// - 대시보드 그리드 데이터 + 가상 포지션 + 알림 설정
#[tauri::command]
pub fn export_csv(
    db: State<'_, Database>,
    file_path: String,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // KST 시간 생성
    let kst = FixedOffset::east_opt(9 * 3600).unwrap();
    let now_kst = chrono::Utc::now().with_timezone(&kst);
    let export_time = now_kst.format("%Y-%m-%d %H:%M:%S KST").to_string();

    // CSV 헤더
    let header = "종목명,종목코드,시장구분,현재가,전일종가,증감액,등락률(%),거래량,지수대비등락률차이,매수일,평균단가,수량,평가손익,수익률(%),목표가,손절가,데이터상태,내보낸시간";

    let mut csv_lines: Vec<String> = Vec::new();
    csv_lines.push(header.to_string());

    // 즐겨찾기 목록 조회
    let favorites = crate::db::models::get_favorites(&conn).map_err(|e| e.to_string())?;

    // 가상 포지션 맵 조회
    let positions = crate::db::models::get_positions(&conn).map_err(|e| e.to_string())?;
    let pos_map: std::collections::HashMap<String, &VirtualPosition> = positions
        .iter()
        .map(|p| (p.ticker.clone(), p))
        .collect();

    // 알림 설정 맵 조회
    let alerts = crate::db::models::get_alerts(&conn).map_err(|e| e.to_string())?;
    let alert_map: std::collections::HashMap<String, &AlertConfig> = alerts
        .iter()
        .map(|a| (a.ticker.clone(), a))
        .collect();

    for fav in &favorites {
        // 포지션 정보 (있을 경우)
        let pos = pos_map.get(&fav.ticker);
        let buy_date = pos.and_then(|p| p.buy_date.as_deref()).unwrap_or("");
        let avg_price = pos.and_then(|p| p.avg_buy_price).map(|v| v.to_string()).unwrap_or("".to_string());
        let quantity = pos.and_then(|p| p.quantity).map(|v| v.to_string()).unwrap_or("".to_string());

        // 평가손익/수익률 계산 (실제 시세 연동 전이므로 0)
        let eval_profit = "";
        let eval_profit_pct = "";

        // 알림 설정 (목표가/손절가)
        let alert = alert_map.get(&fav.ticker);
        let target_price = if let Some(a) = alert {
            if a.alert_type == AlertType::Target {
                a.threshold.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        let stop_loss_price = if let Some(a) = alert {
            if a.alert_type == AlertType::StopLoss {
                a.threshold.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        // CSV 라인 생성 (모든 값은 큰따옴표로 감싸서 안전하게)
        let line = format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
            csv_escape(&fav.name),
            fav.ticker,
            csv_escape(&fav.market_section),
            "",  // 현재가 (시세 연동 전)
            "",  // 전일종가
            "",  // 증감액
            "",  // 등락률
            "",  // 거래량
            "",  // 지수대비등락률차이
            buy_date,
            avg_price,
            quantity,
            eval_profit,
            eval_profit_pct,
            target_price,
            stop_loss_price,
            "stale", // 데이터 상태 (시세 연동 전)
            export_time,
        );
        csv_lines.push(line);
    }

    // UTF-8 BOM 추가 (Excel 호환)
    let mut csv_content = String::from("\u{FEFF}");
    csv_content.push_str(&csv_lines.join("\n"));

    // 파일 저장
    std::fs::write(&file_path, csv_content.as_bytes())
        .map_err(|e| format!("Failed to write CSV: {}", e))?;

    tracing::info!("CSV exported to {}", file_path);
    Ok(file_path)
}

/// CSV 필드 이스케이프 (큰따옴표 포함 시 두 번 반복)
fn csv_escape(s: &str) -> String {
    s.replace('"', "\"\"")
}

// === Naver ETF List & Manual Quote (REQ-F-17 확장) ===

/// 네이버 전체 ETF 목록 스크래핑 (sise_market_sum.naver?menu=etf)
/// 폴링 상태와 무관하게 전체 ETF 종목 리스트를 조회한다.
#[tauri::command]
pub async fn fetch_naver_etf_list(
    provider_mgr: State<'_, tokio::sync::Mutex<ProviderManager>>,
) -> Result<Vec<EtfListItem>, String> {
    // tokio::sync::Mutex guard는 Send이므로 await across lock 안전
    let mut mgr = provider_mgr.lock().await;
    match mgr.fetch_etf_list().await {
        Ok(items) => Ok(items),
        Err(e) => {
            let message = format!("fetch_naver_etf_list failed: {:#}", e);
            tracing::error!("{}", message);
            Err(message)
        }
    }
}

/// 수동 즉시 시세 조회 — 장 마감/휴장일에도 동작
/// 폴링 상태와 무관하게 ProviderManager를 통해 즉시 시세를 가져온다.
#[tauri::command]
pub async fn fetch_quote_now(
    provider_mgr: State<'_, tokio::sync::Mutex<ProviderManager>>,
    ticker: String,
) -> Result<NormalizedQuote, String> {
    let mut mgr = provider_mgr.lock().await;
    mgr.fetch_quote(&ticker)
        .await
        .map_err(|e| e.to_string())
}
