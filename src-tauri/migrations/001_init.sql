-- 001_init.sql — ETF Tracker 초기 스키마
-- REQ-F-14: 즐겨찾기, 가상 포지션, 알림 설정, 앱 재시작 시 복원

-- 즐겨찾기 종목
CREATE TABLE IF NOT EXISTS favorites (
    ticker TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    market_section TEXT,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_active INTEGER NOT NULL DEFAULT 1
);

-- 가상 포지션
CREATE TABLE IF NOT EXISTS virtual_positions (
    ticker TEXT PRIMARY KEY REFERENCES favorites(ticker) ON DELETE CASCADE,
    buy_date TEXT,
    avg_buy_price REAL,
    quantity REAL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 알림 설정
CREATE TABLE IF NOT EXISTS alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticker TEXT NOT NULL REFERENCES favorites(ticker) ON DELETE CASCADE,
    alert_type TEXT NOT NULL,     -- 'target' | 'stop_loss'
    threshold REAL NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    triggered_at TEXT,
    triggered_price REAL,
    reset_at TEXT
);

-- 알림 히스토리 (P1)
CREATE TABLE IF NOT EXISTS alert_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticker TEXT NOT NULL,
    alert_type TEXT NOT NULL,
    threshold REAL,
    triggered_price REAL,
    triggered_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 앱 설정
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 기본 설정값
INSERT OR IGNORE INTO settings (key, value) VALUES ('polling_interval', '60');
INSERT OR IGNORE INTO settings (key, value) VALUES ('minimize_to_tray', 'true');
INSERT OR IGNORE INTO settings (key, value) VALUES ('first_close_notice', 'false');

-- 일간 거래 데이터 캐시
CREATE TABLE IF NOT EXISTS daily_quotes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticker TEXT NOT NULL,
    date TEXT NOT NULL,
    open REAL,
    high REAL,
    low REAL,
    close REAL,
    volume INTEGER,
    UNIQUE(ticker, date)
);

-- 인덱스
CREATE INDEX IF NOT EXISTS idx_alerts_ticker ON alerts(ticker);
CREATE INDEX IF NOT EXISTS idx_daily_quotes_ticker_date ON daily_quotes(ticker, date);
CREATE INDEX IF NOT EXISTS idx_alert_history_ticker ON alert_history(ticker);