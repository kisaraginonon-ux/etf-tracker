// Normalizer — Provider 응답 정규화 및 검증
// 각 Provider의 응답을 표준 NormalizedQuote로 매핑 후 검증

use crate::models::NormalizedQuote;

/// 정규화된 시세의 유효성 검사
/// - 필수 필드 누락 시 ProviderError::MissingField 반환
pub fn validate_quote(quote: &NormalizedQuote) -> Result<(), NormalizationError> {
    if quote.ticker.is_empty() {
        return Err(NormalizationError::MissingField("ticker"));
    }
    if quote.current_price <= 0.0 {
        return Err(NormalizationError::InvalidValue("current_price"));
    }
    if quote.name.is_empty() {
        return Err(NormalizationError::MissingField("name"));
    }
    Ok(())
}

/// 배치 시세 검사 — 유효한 것만 필터링
pub fn filter_valid_quotes(quotes: Vec<NormalizedQuote>) -> Vec<NormalizedQuote> {
    quotes.into_iter()
        .filter(|q| validate_quote(q).is_ok())
        .collect()
}

/// 정규화 오류
#[derive(Debug, thiserror::Error)]
pub enum NormalizationError {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
    #[error("Invalid value for {0}")]
    InvalidValue(&'static str),
}