// ETF Tracker — Library Root
// 모든 모듈의 중앙 진입점

mod alert;
mod calendar;
mod commands;
mod db;
mod logging;
mod models;
mod notification;
mod polling;
mod providers;

use std::sync::Mutex;
use tauri::Manager;

use calendar::MarketCalendar;
use polling::PollingManager;
use providers::{NaverProvider, YahooProvider, ProviderManager};

/// 애플리케이션 진입점 — Tauri 앱 초기화 및 IPC Commands 등록
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 로깅 시스템 초기화
    logging::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // SQLite 데이터베이스 초기화
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).ok();
            let db_path = app_data_dir.join("etf_tracker.db");
            let database = db::Database::new(&db_path)?;
            database.migrate()?;
            app.manage(database);

            // Market Calendar 초기화 — 캘린더 JSON 로드 시도
            let mut market_cal = MarketCalendar::new();
            let cal_path = "src-tauri/data/market_calendar.json";
            if std::path::Path::new(cal_path).exists() {
                if let Err(e) = market_cal.load_holidays(cal_path) {
                    tracing::warn!("Failed to load calendar: {} — using Fallback mode", e);
                }
            } else {
                tracing::warn!("Calendar JSON not found at {} — using Fallback mode", cal_path);
            }
            app.manage(Mutex::new(market_cal));

            // Provider Manager 초기화 (Naver Primary, Yahoo Fallback)
            // tokio::sync::Mutex 사용 — async 명령에서 await across lock 필요 (Send guard)
            let provider_manager = ProviderManager::new(
                Box::new(NaverProvider::new()),
                Box::new(YahooProvider::new()),
            );
            app.manage(tokio::sync::Mutex::new(provider_manager));

            // Polling Manager 초기화
            let polling_cal = MarketCalendar::new();
            let polling_mgr = PollingManager::new(polling_cal);
            app.manage(Mutex::new(polling_mgr));

            tracing::info!("ETF Tracker started — DB at {:?}", db_path);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Calendar
            commands::get_market_state,
            commands::get_market_progress,
            // ETF Master
            commands::search_etf,
            // Favorites
            commands::get_favorites,
            commands::add_favorite,
            commands::remove_favorite,
            // Virtual Positions
            commands::set_position,
            commands::get_positions,
            commands::remove_position,
            // Alerts
            commands::set_alert,
            commands::get_alerts,
            commands::reset_alert,
            commands::remove_alert,
            // Polling
            commands::set_polling_interval,
            commands::pause_polling,
            commands::resume_polling,
            commands::get_polling_status,
            commands::manual_refresh,
            // CSV Export
            commands::export_csv,
            // Provider Status
            commands::get_provider_status,
            // Settings
            commands::get_setting,
            commands::set_setting,
            // Naver ETF List & Manual Quote
            commands::fetch_naver_etf_list,
            commands::fetch_quote_now,
            commands::fetch_period_returns,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ETF Tracker");
}