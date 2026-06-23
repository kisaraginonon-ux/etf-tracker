// Models — 표준 데이터 모델
// NormalizedQuote 및 관련 타입 정의

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Provider 식별자
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderName {
    Naver,
    Yahoo,
}

impl std::fmt::Display for ProviderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderName::Naver => write!(f, "naver"),
            ProviderName::Yahoo => write!(f, "yahoo"),
        }
    }
}

/// 정규화된 시세 데이터 (Provider 정규화 후 표준 모델)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedQuote {
    pub ticker: String,
    pub name: String,
    pub current_price: f64,
    pub prev_close: f64,
    pub change: f64,       // 전일 대비 증감액
    pub change_pct: f64,   // 전일 대비 등락률 (%)
    pub volume: u64,       // 당일 누적 거래량
    pub timestamp: DateTime<Utc>,
    pub provider: ProviderName,
}

/// 장중 시점별 가격 데이터 (스파크라인용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntradayPoint {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub volume: u64,
}

/// 시장 지수 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketIndex {
    pub name: String,
    pub code: String,
    pub current_price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub timestamp: DateTime<Utc>,
}

/// ETF 종목 마스터 항목
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtfMasterItem {
    pub ticker: String,       // 단축코드
    pub name: String,         // 종목명
    pub market_section: String, // 시장구분
    pub is_active: bool,      // 상장 상태
}

/// 네이버 전체 ETF 목록 스크래핑 결과 항목
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtfListItem {
    pub ticker: String,       // 6자리 종목코드
    pub name: String,         // 종목명
    pub current_price: f64,   // 현재가
    pub change_pct: f64,      // 등락률 (%)
    pub volume: u64,          // 거래량
    pub prev_close: f64,      // 전일종가
    pub change_amount: f64,   // 등락액 (현재가 - 전일종가)
    pub trading_value: u64,   // 거래대금
}

/// 기간별 등락률
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodReturn {
    pub period: String,       // "1d", "1w", "1m", "3m", "6m", "1y"
    pub label: String,        // "1일", "1주", "1개월", "3개월", "6개월", "1년"
    pub return_pct: f64,      // 등락률 (%)
    pub start_price: f64,     // 시작가
    pub end_price: f64,       // 종료가
}

/// 기간별 등락률 응답
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodReturns {
    pub ticker: String,
    pub name: String,
    pub current_price: f64,
    pub volume: u64,
    pub returns: Vec<PeriodReturn>,
}

/// 모멘텀 시그널
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumSignal {
    pub ticker: String,
    pub volume_ratio: f64,   // 시그널 배율 = 현재 누적 / 기대 누적
    pub state: SignalState,
    pub is_low_liquidity: bool, // 저유동성 (거래대금 < 1억)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignalState {
    Provisional, // 장중 "잠정"
    Confirmed,   // 장 종료 후 "확정"
}

/// 데이터 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataStatus {
    Live,           // 정규장 중, 최근 갱신 ≤ 3분
    Stale,          // 정규장 중, 최근 갱신 > 3분
    MarketClosed,   // 장 종료
    PreMarket,      // 장전
    Holiday,        // 휴장일
    ProviderError,  // Provider 장애
}

/// 가상 포지션
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualPosition {
    pub ticker: String,
    pub buy_date: Option<String>,
    pub avg_buy_price: Option<f64>,
    pub quantity: Option<f64>,
}

/// 알림 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub ticker: String,
    pub alert_type: AlertType,
    pub threshold: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertType {
    Target,   // 목표가 (상한)
    StopLoss, // 손절가 (하한)
}

/// 즐겨찾기 종목
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub ticker: String,
    pub name: String,
    pub market_section: String,
    pub added_at: String,
    pub is_active: bool,
}