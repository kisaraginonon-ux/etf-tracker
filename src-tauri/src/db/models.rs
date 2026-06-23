// DB Models — 쿼리용 데이터 변환

use rusqlite::{params, Connection};
use anyhow::Result;

use crate::models::{Favorite, VirtualPosition, AlertConfig, AlertType};

impl Favorite {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            ticker: row.get("ticker")?,
            name: row.get("name")?,
            market_section: row.get("market_section")?,
            added_at: row.get("added_at")?,
            is_active: row.get("is_active")?,
        })
    }
}

/// 즐겨찾기 CRUD
pub fn add_favorite(conn: &Connection, ticker: &str, name: &str, market_section: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO favorites (ticker, name, market_section) VALUES (?1, ?2, ?3)",
        params![ticker, name, market_section],
    )?;
    Ok(())
}

pub fn remove_favorite(conn: &Connection, ticker: &str) -> Result<()> {
    conn.execute("DELETE FROM favorites WHERE ticker = ?1", params![ticker])?;
    Ok(())
}

pub fn get_favorites(conn: &Connection) -> Result<Vec<Favorite>> {
    let mut stmt = conn.prepare(
        "SELECT ticker, name, market_section, added_at, is_active FROM favorites WHERE is_active = 1 ORDER BY added_at"
    )?;
    let favorites = stmt.query_map([], Favorite::from_row)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(favorites)
}

/// 가상 포지션 CRUD
pub fn set_position(conn: &Connection, ticker: &str, buy_date: Option<&str>, avg_price: Option<f64>, qty: Option<f64>) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO virtual_positions (ticker, buy_date, avg_buy_price, quantity) VALUES (?1, ?2, ?3, ?4)",
        params![ticker, buy_date, avg_price, qty],
    )?;
    Ok(())
}

pub fn get_positions(conn: &Connection) -> Result<Vec<VirtualPosition>> {
    let mut stmt = conn.prepare(
        "SELECT ticker, buy_date, avg_buy_price, quantity FROM virtual_positions"
    )?;
    let positions = stmt.query_map([], |row| {
        Ok(VirtualPosition {
            ticker: row.get(0)?,
            buy_date: row.get(1)?,
            avg_buy_price: row.get(2)?,
            quantity: row.get(3)?,
        })
    })?.filter_map(|r| r.ok()).collect();
    Ok(positions)
}

/// 알림 CRUD
pub fn set_alert(conn: &Connection, ticker: &str, alert_type: &str, threshold: f64) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO alerts (ticker, alert_type, threshold, is_active) VALUES (?1, ?2, ?3, 1)",
        params![ticker, alert_type, threshold],
    )?;
    Ok(())
}

pub fn get_alerts(conn: &Connection) -> Result<Vec<AlertConfig>> {
    let mut stmt = conn.prepare(
        "SELECT ticker, alert_type, threshold, is_active FROM alerts WHERE is_active = 1"
    )?;
    let alerts = stmt.query_map([], |row| {
        let alert_type_str: String = row.get(1)?;
        let alert_type = match alert_type_str.as_str() {
            "target" => AlertType::Target,
            "stop_loss" => AlertType::StopLoss,
            _ => AlertType::Target,
        };
        Ok(AlertConfig {
            ticker: row.get(0)?,
            alert_type,
            threshold: row.get(2)?,
            is_active: row.get(3)?,
        })
    })?.filter_map(|r| r.ok()).collect();
    Ok(alerts)
}