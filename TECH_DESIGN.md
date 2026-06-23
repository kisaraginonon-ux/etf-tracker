# ETF Tracker — 기술 설계서 (Technical Design Document)

> 본 문서는 PRD(ETF_Tracker_PRD_Final1)의 요구사항을 구현하기 위한 기술 아키텍처, 모듈 설계, 데이터 모델, API 정규화, 에러 처리 정책을 정의한다.

---

## 1. 시스템 아키텍처 개요

```
┌─────────────────────────────────────────────────────┐
│                    Tauri Application                 │
│                                                      │
│  ┌─────────────┐     IPC (Tauri Commands)           │
│  │   Frontend   │◄──────────────────────────────────►│
│  │  (Svelte +   │     invoke('get_quotes', ...)      │
│  │  Chart.js)   │     invoke('add_favorite', ...)    │
│  └──────┬──────┘                                    │
│         │                                            │
│  ┌──────▼───────────────────────────────────────────│
│  │                 Rust Backend                      │
│  │                                                   │
│  │  ┌──────────┐  ┌───────────┐  ┌───────────────┐  │
│  │  │  Data     │  │  Market   │  │  Notification  │  │
│  │  │  Provider │  │  Calendar │  │  (Toast/Tray)  │  │
│  │  │  Layer    │  │  Engine   │  │  Manager       │  │
│  │  └────┬─────┘  └─────┬─────┘  └───────────────┘  │
│  │       │               │                          │
│  │  ┌────▼───────────────▼──────────────────────┐   │
│  │  │              Core Engine (tokio)           │   │
│  │  │  ┌─────────┐  ┌──────────┐  ┌──────────┐  │   │
│  │  │  │  Polling │  │  Quote   │  │  Alert   │  │   │
│  │  │  │  Manager │  │  Normal  │  │  Engine  │  │   │
│  │  │  │  (Backoff│  │  izer    │  │(Debounce)│  │   │
│  │  │  │  Policy) │  │          │  │          │  │   │
│  │  │  └─────────┘  └──────────┘  └──────────┘  │   │
│  │  └────────────────────────────────────────────┘  │
│  │                                                   │
│  │  ┌─────────────────────────────────────────────┐ │
│  │  │                Storage Layer                  │ │
│  │  │  ┌──────────┐  ┌────────────┐  ┌──────────┐ │ │
│  │  │  │  SQLite  │  │  JSON      │  │  Logger   │ │ │
│  │  │  │(rusqlite)│  │  Masters   │  │(rotation) │ │ │
│  │  │  └──────────┘  └────────────┘  └──────────┘ │ │
│  │  └─────────────────────────────────────────────┘ │
│  └───────────────────────────────────────────────────│
└─────────────────────────────────────────────────────┘
```

## 2. 기술 스택 상세

| 레이어 | 기술 | 비고 |
|--------|------|------|
| **Framework** | Tauri v2 | Rust backend + WebView2 frontend |
| **Frontend** | Svelte 5 + Vite | 반응형 UI, 작은 번들 |
| **Charting** | uPlot | 경량 고성능 (Sparkline, 시계열) |
| **Backend** | Rust (stable) | reqwest, rusqlite, tokio, serde |
| **DB** | SQLite (rusqlite) | 로컬 영구 저장, 마이그레이션 |
| **Notification** | Tauri notification + tray plugin | Windows Toast, 시스템 트레이 |
| **HTTP** | reqwest (async) | Provider API 호출 |
| **Serialization** | serde + serde_json | JSON 마스터 데이터, API 응답 |
| **Logging** | tracing + tracing-appender | 14일 rotation, 민감정보 마스킹 |
| **Installer** | NSIS (Tauri 기본) | Windows 10/11 타겟 |

## 3. 모듈 설계

### 3.1 Data Provider Layer

```
src-tauri/src/providers/
├── mod.rs          — Provider trait 정의 및 fallback 로직
├── naver.rs        — Naver Finance API (Primary)
├── yahoo.rs        — Yahoo Finance API (Fallback)
└── normalizer.rs   — 표준 Quote 모델 정규화
```

**Provider Trait:**
```rust
#[async_trait]
pub trait DataProvider: Send + Sync {
    async fn fetch_quote(&self, ticker: &str) -> Result<NormalizedQuote>;
    async fn fetch_batch(&self, tickers: &[&str]) -> Result<Vec<NormalizedQuote>>;
    async fn fetch_index(&self, index_code: &str) -> Result<NormalizedQuote>;
    async fn fetch_intraday(&self, ticker: &str) -> Result<Vec<IntradayPoint>>;
    async fn health_check(&self) -> Result<bool>;
    fn name(&self) -> &str;
}
```

**NormalizedQuote (표준 모델):**
```rust
pub struct NormalizedQuote {
    pub ticker: String,
    pub name: String,
    pub current_price: f64,
    pub prev_close: f64,
    pub change: f64,        // 전일 대비 증감액
    pub change_pct: f64,     // 전일 대비 등락률 (%)
    pub volume: u64,         // 당일 누적 거래량
    pub timestamp: DateTime<Utc>,
    pub provider: ProviderName,
}
```

### 3.2 Market Calendar Engine

```
src-tauri/src/calendar/
├── mod.rs          — 장 상태 판별 엔진
├── schedule.rs     — 정규장 시간 (09:00~15:30 KST)
└── holidays.rs     — 휴장일 로직 (JSON 캘린더 기반)
```

**MarketState:**
```rust
pub enum MarketState {
    PreOpen,    // 장전 (08:00~09:00)
    Regular,    // 정규장 (09:00~15:30)
    Closed,     // 장 종료
    Holiday,    // 휴장일
    Unknown,    // 캘린더 누락 → Fallback 모드
}
```

### 3.3 Polling Manager

```rust
pub struct PollingManager {
    interval: Duration,          // 1/3/5분
    backoff: ExponentialBackoff,  // API 장애 시
    market_calendar: MarketCalendar,
    paused: bool,                // 일시정지 상태
}

// 정규장 중에만 자동 폴링, 장외는 수동만 허용
// 백오프: 1min → 2min → 4min → 8min → max 30min
```

### 3.4 Alert Engine (Debounce)

```rust
pub struct AlertEngine {
    alerts: HashMap<Ticker, AlertState>,
}

pub struct AlertState {
    target_price: Option<f64>,   // 목표가 (상한)
    stop_loss: Option<f64>,       // 손절가 (하한)
    target_triggered: bool,      // 이미 발송했는가
    stop_triggered: bool,
}

// 조건 해소(가격 복귀) 후 재돌파 시 재발송
// Stale/장애/장종료 상태에서는 발송 중단
```

### 3.5 Notification Manager

```rust
pub struct NotificationManager {
    tray: TrayIcon,
    toast: ToastHandler,
}

// REQ-F-10: 트레이 컨텍스트 메뉴 [열기, 일시정지, 새로고침, 종료]
// REQ-F-11: 목표가/손절가 돌파 시 토스트 1회
// REQ-F-13: Stale/장애/장종료 시 알림 중단
```

## 4. 데이터 모델 (SQLite Schema)

```sql
-- 스키마 버전 관리
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 즐겨찾기 종목
CREATE TABLE favorites (
    ticker TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    market_section TEXT,          -- 시장구분
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_active INTEGER NOT NULL DEFAULT 1
);

-- 가상 포지션
CREATE TABLE virtual_positions (
    ticker TEXT PRIMARY KEY REFERENCES favorites(ticker),
    buy_date TEXT,
    avg_buy_price REAL,
    quantity REAL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 알림 설정
CREATE TABLE alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticker TEXT NOT NULL REFERENCES favorites(ticker),
    alert_type TEXT NOT NULL,     -- 'target' | 'stop_loss'
    threshold REAL NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    triggered_at TEXT,            -- 마지막 발송 시각
    triggered_price REAL,         -- 발송 시 가격
    reset_at TEXT                 -- 재설정 시각
);

-- 알림 히스토리 (P1)
CREATE TABLE alert_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticker TEXT NOT NULL,
    alert_type TEXT NOT NULL,
    threshold REAL,
    triggered_price REAL,
    triggered_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 앱 설정
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- 기본값: polling_interval=60, minimize_to_tray=true, ...

-- 일간 거래 데이터 캐시 (과거 데이터)
CREATE TABLE daily_quotes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticker TEXT NOT NULL,
    date TEXT NOT NULL,
    open REAL, high REAL, low REAL, close REAL,
    volume INTEGER,
    UNIQUE(ticker, date)
);
```

## 5. API Provider 정규화 및 Fallback 정책

### 5.1 Provider 우선순위
1. **Primary**: Naver Finance API
2. **Fallback**: Yahoo Finance API

### 5.2 Fallback 전환 조건 (REQ-F-19)
- Primary 3회 연속 호출 실패 → Fallback 전환
- Primary 필수 필드 2회 연속 누락 → Fallback 전환
- 10분 주기 Primary 복구 시도

### 5.3 정규화 흐름
```
Naver Response ─┐
                ├─→ normalizer.rs ─→ NormalizedQuote
Yahoo Response ──┘
```

각 Provider의 응답 필드를 `NormalizedQuote`로 매핑:
- 필수 필드 누락 시 `ProviderError::MissingField`
- 정규화 실패 시 해당 종목 `Stale` 처리

## 6. 데이터 상태 관리 (REQ-F-18)

| 상태 | 조건 | 표시 |
|------|------|------|
| **Live** | 정규장 중, 마지막 갱신 ≤ 3분 | 정상 |
| **Stale** | 정규장 중, 마지막 갱신 > 3분 | "Stale" 배지 |
| **Market Closed** | 장 종료 후 | "장마감 데이터" |
| **Pre-Market** | 장전 | "시장 미운영" |
| **Holiday** | 휴장일 | "시장 미운영" |
| **Provider Error** | Provider 장애 | "데이터 오류" |

## 7. 모멘텀 시그널 로직 (REQ-F-09)

```
기대 누적 거래량 = 5거래일 평균 일거래량 × (경과 분 / 전체 정규장 분)
시그널 배율 = 현재 누적 거래량 / 기대 누적 거래량

예외:
- 대체 지표: 과거 장중 시간대별 거래량 확보 불가 시
  → 일평균 거래량 대비 현재 누적의 시간 비례 기대치
- 저유동성 배제: 5거래일 평균 거래대금 < 1억 → "저유동성 주의" 배지
- 상태 분리: 장중 "잠정", 장 종료 후 "확정"
```

## 8. 로깅 정책 (REQ-NF-02)

- **Level**: tracing crate 사용 (INFO/WARN/ERROR)
- **Rotation**: 14일 보관 후 순환 삭제 (tracing-appender)
- **마스킹**: 가상 포지션 정보(매수단가, 수량)는 로그에서 제외
- **기록 항목**: API 실패, Fallback 전환, Stale 전환, 알림 발송

## 9. 프로젝트 구조

```
etf-tracker/
├── src-tauri/                  # Rust 백엔드
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs             # 진입점
│   │   ├── commands.rs          # Tauri IPC Commands
│   │   ├── providers/
│   │   │   ├── mod.rs
│   │   │   ├── naver.rs
│   │   │   ├── yahoo.rs
│   │   │   └── normalizer.rs
│   │   ├── calendar/
│   │   │   ├── mod.rs
│   │   │   ├── schedule.rs
│   │   │   └── holidays.rs
│   │   ├── polling/
│   │   │   └── mod.rs            # PollingManager + Backoff
│   │   ├── alert/
│   │   │   └── mod.rs            # AlertEngine + Debounce
│   │   ├── notification/
│   │   │   └── mod.rs            # Toast + Tray
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs         # 마이그레이션
│   │   │   └── models.rs
│   │   ├── models/
│   │   │   └── mod.rs            # NormalizedQuote 등
│   │   └── logging/
│   │       └── mod.rs
│   ├── migrations/
│   │   └── 001_init.sql
│   └── data/
│       ├── etf_master.json      # ETF 종목 마스터
│       └── market_calendar.json  # 시장 캘린더
│
├── src/                         # Svelte Frontend
│   ├── App.svelte
│   ├── lib/
│   │   ├── components/
│   │   │   ├── Dashboard.svelte      # 메인 그리드
│   │   │   ├── QuoteGrid.svelte      # 데이터 그리드
│   │   │   ├── Sparkline.svelte      # 장중 스파크라인
│   │   │   ├── PositionPanel.svelte  # 가상 포지션
│   │   │   ├── AlertSettings.svelte  # 알림 설정
│   │   │   └── StatusBadge.svelte    # 상태 배지
│   │   ├── stores/
│   │   │   ├── quotes.ts             # 현재가 스토어
│   │   │   ├── favorites.ts          # 즐겨찾기 스토어
│   │   │   ├── positions.ts         # 포지션 스토어
│   │   │   └── settings.ts           # 설정 스토어
│   │   └── api/
│   │       └── tauri.ts              # Tauri invoke 래퍼
│   ├── routes/
│   └── app.html
│
├── static/
│   └── icons/
├── package.json
├── vite.config.ts
├── svelte.config.js
├── tsconfig.json
├── TECH_DESIGN.md               # 본 문서
├── Plan.md                       # 개발 계획
└── README.md
```

## 10. Tauri IPC Commands (Frontend ↔ Backend)

| Command | 파라미터 | 반환 | 설명 |
|---------|----------|------|------|
| `get_quotes` | tickers: string[] | Quote[] | 현재가 일괄 조회 |
| `get_index` | index_code: string | Quote | 시장 지수 조회 |
| `add_favorite` | ticker: string | void | 즐겨찾기 추가 |
| `remove_favorite` | ticker: string | void | 즐겨찾기 제거 |
| `get_favorites` | — | Favorite[] | 즐겨찾기 목록 |
| `set_position` | ticker, price, qty, date | void | 가상 포지션 설정 |
| `get_positions` | — | Position[] | 포지션 목록 |
| `set_alert` | ticker, type, threshold | void | 알림 설정 |
| `get_alerts` | — | Alert[] | 알림 목록 |
| `set_polling_interval` | seconds: int | void | 갱신 주기 변경 |
| `pause_polling` | — | void | 일시정지 |
| `resume_polling` | — | void | 재개 |
| `manual_refresh` | — | Quote[] | 수동 새로고침 |
| `export_csv` | scope: string | string | CSV 내보내기 |
| `search_etf` | query: string | ETF[] | 종목 검색 |
| `get_market_state` | — | MarketState | 장 상태 |
| `get_signal` | ticker: string | Signal | 모멘텀 시그널 |