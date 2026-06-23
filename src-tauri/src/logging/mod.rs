// Logging — tracing + tracing-appender
// REQ-NF-02: 14일 rotation, 민감정보 마스킹

use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// 로깅 시스템 초기화
/// - 로그 위치: 앱 데이터 디렉토리/logs/
/// - 14일 rotation (daily)
/// - 민감정보(포지션 등)는 마스킹 — 각 모듈에서 log 호출 시 주의
pub fn init() {
    // 로그 디렉토리 — 임시로 현재 디렉토리 사용, setup에서 app_data_dir로 교체 가능
    let log_dir = std::env::var("ETF_TRACKER_LOG_DIR").unwrap_or_else(|_| "logs".to_string());
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = rolling::daily(&log_dir, "etf-tracker.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
        )
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true)
        )
        .init();

    // guard를 leak하여 프로세스 종료 시까지 유지
    std::mem::forget(_guard);

    tracing::info!("Logging initialized — log dir: {}", log_dir);
}

/// 14일 이전 로그 파일 정리 (startup 시 호출)
pub fn cleanup_old_logs(log_dir: &str) {
    let retention_days = chrono::Duration::days(14);
    let now = chrono::Utc::now();

    if let Ok(entries) = std::fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified_time: chrono::DateTime<chrono::Utc> = modified.into();
                    if now - modified_time > retention_days {
                        let _ = std::fs::remove_file(entry.path());
                        tracing::info!("Rotated old log: {:?}", entry.path());
                    }
                }
            }
        }
    }
}