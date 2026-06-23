// Notification Manager — Toast + System Tray
// REQ-F-10: 시스템 트레이 최소화, 컨텍스트 메뉴
// REQ-F-11: Windows Toast 알림

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub struct NotificationManager;

impl NotificationManager {
    /// Windows Toast 알림 발송
    pub fn send_toast(app: &AppHandle, title: &str, body: &str) {
        if let Err(e) = app.notification().builder()
            .title(title)
            .body(body)
            .show()
        {
            tracing::error!("Failed to send toast notification: {}", e);
        }
    }

    /// 목표가 돌파 알림
    pub fn notify_target_hit(app: &AppHandle, ticker: &str, name: &str, price: f64, target: f64) {
        let title = "목표가 도달";
        let body = format!("{}({}) 현재가 {}원 — 목표가 {}원 도달!", name, ticker, price, target);
        Self::send_toast(app, title, &body);
        tracing::info!("Target alert sent: {} @ {} (target: {})", ticker, price, target);
    }

    /// 손절가 돌파 알림
    pub fn notify_stop_loss_hit(app: &AppHandle, ticker: &str, name: &str, price: f64, stop: f64) {
        let title = "손절가 도달";
        let body = format!("{}({}) 현재가 {}원 — 손절가 {}원 도달!", name, ticker, price, stop);
        Self::send_toast(app, title, &body);
        tracing::info!("Stop loss alert sent: {} @ {} (stop: {})", ticker, price, stop);
    }
}