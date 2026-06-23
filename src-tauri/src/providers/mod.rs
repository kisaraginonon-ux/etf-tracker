// Providers — Data Provider Layer
// REQ-F-17: Naver(Primary) → Yahoo(Fallback), 정규화

pub mod naver;
pub mod yahoo;
pub mod normalizer;

pub use naver::NaverProvider;
pub use yahoo::YahooProvider;

use async_trait::async_trait;
use anyhow::Result;

use crate::models::{NormalizedQuote, IntradayPoint, ProviderName, EtfListItem};

/// Provider Trait — 모든 데이터 프로바이더가 구현해야 하는 인터페이스
#[async_trait]
pub trait DataProvider: Send + Sync {
    async fn fetch_quote(&self, ticker: &str) -> Result<NormalizedQuote>;
    async fn fetch_batch(&self, tickers: &[&str]) -> Result<Vec<NormalizedQuote>>;
    async fn fetch_index(&self, index_code: &str) -> Result<NormalizedQuote>;
    async fn fetch_intraday(&self, ticker: &str) -> Result<Vec<IntradayPoint>>;
    async fn health_check(&self) -> Result<bool>;
    fn name(&self) -> &str;
    fn provider_name(&self) -> ProviderName;

    /// 네이버 전체 ETF 목록 스크래핑 — NaverProvider만 구현, 기본은 빈 벡터
    async fn fetch_etf_list(&self) -> Result<Vec<EtfListItem>> {
        Ok(Vec::new())
    }
}

/// Provider 오류
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Unknown ticker: {0}")]
    UnknownTicker(String),
}

/// Fallback 매니저 — Primary/Fallback 전환 로직 (REQ-F-19)
pub struct ProviderManager {
    primary: Box<dyn DataProvider>,
    fallback: Box<dyn DataProvider>,
    primary_failures: u32,
    field_failures: u32,
    using_fallback: bool,
    last_primary_retry: Option<chrono::DateTime<chrono::Utc>>,
}

impl ProviderManager {
    pub fn new(primary: Box<dyn DataProvider>, fallback: Box<dyn DataProvider>) -> Self {
        Self {
            primary,
            fallback,
            primary_failures: 0,
            field_failures: 0,
            using_fallback: false,
            last_primary_retry: None,
        }
    }

    /// 10분 주기 Primary 복구 시도 (REQ-F-19)
    fn should_retry_primary(&self) -> bool {
        if !self.using_fallback {
            return false;
        }
        match self.last_primary_retry {
            None => true,
            Some(last) => chrono::Utc::now() - last > chrono::Duration::minutes(10),
        }
    }

    /// 현재 활성 Provider 이름
    pub fn active_provider(&self) -> ProviderName {
        if self.using_fallback {
            self.fallback.provider_name()
        } else {
            self.primary.provider_name()
        }
    }

    /// Fallback 모드 여부
    pub fn is_using_fallback(&self) -> bool {
        self.using_fallback
    }

    /// Primary 연속 실패 횟수
    pub fn primary_failures(&self) -> u32 {
        self.primary_failures
    }

    /// 현재 활성 Provider로부터 시세 조회
    pub async fn fetch_quote(&mut self, ticker: &str) -> Result<NormalizedQuote> {
        if self.using_fallback && self.should_retry_primary() {
            tracing::info!("Attempting primary provider recovery...");
            if self.primary.health_check().await.unwrap_or(false) {
                tracing::info!("Primary provider recovered, switching back");
                self.using_fallback = false;
                self.primary_failures = 0;
                self.field_failures = 0;
            }
            self.last_primary_retry = Some(chrono::Utc::now());
        }

        if !self.using_fallback {
            match self.primary.fetch_quote(ticker).await {
                Ok(quote) => {
                    self.primary_failures = 0;
                    Ok(quote)
                }
                Err(e) => {
                    self.primary_failures += 1;
                    tracing::warn!("Primary failed ({}): {} — failures: {}", ticker, e, self.primary_failures);
                    if self.primary_failures >= 3 {
                        tracing::warn!("Switching to fallback provider (3 consecutive failures)");
                        self.using_fallback = true;
                        self.last_primary_retry = Some(chrono::Utc::now());
                    }
                    self.fallback.fetch_quote(ticker).await
                }
            }
        } else {
            self.fallback.fetch_quote(ticker).await
        }
    }

    /// 네이버 전체 ETF 목록 스크래핑 (Primary Provider 위임)
    /// NaverProvider의 fetch_etf_list를 호출. Fallback(Yahoo)은 ETF 목록 기능이 없으므로
    /// Primary가 Naver일 때만 정상 동작하며, 실패 시 빈 벡터 반환.
    pub async fn fetch_etf_list(&mut self) -> Result<Vec<EtfListItem>> {
        // Primary가 Naver인 경우에만 시도 — fallback 전환 중에도 primary로 시도
        match self.primary.fetch_etf_list().await {
            Ok(items) => Ok(items),
            Err(e) => {
                tracing::warn!("Primary fetch_etf_list failed: {}", e);
                // Fallback은 ETF 목록 스크래핑을 지원하지 않으므로 빈 벡터 반환
                Ok(Vec::new())
            }
        }
    }
}