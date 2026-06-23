// Yahoo Finance Provider (Fallback)
// Yahoo Finance API 기반 국내 ETF 시세 데이터 수집
// 엔드포인트: https://query1.finance.yahoo.com/v8/finance/chart/{ticker}.KS

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;

use crate::models::{NormalizedQuote, IntradayPoint, ProviderName, PeriodReturns, PeriodReturn};
use super::DataProvider;

pub struct YahooProvider {
    client: Client,
}

impl YahooProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    /// 국내 ETF ticker를 Yahoo Finance 형식으로 변환
    /// 069500 → 069500.KS
    fn to_yahoo_ticker(ticker: &str) -> String {
        if ticker.contains('.') {
            ticker.to_string()
        } else {
            format!("{}.KS", ticker)
        }
    }

    /// 기간별 등락률 조회 (Yahoo Finance chart API 사용)
    /// range=1y&interval=1d로 전체 일봉을 가져온 뒤 각 기간의 등락률 계산
    pub async fn fetch_period_returns(&self, ticker: &str) -> Result<PeriodReturns> {
        let yahoo_ticker = Self::to_yahoo_ticker(ticker);
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1y",
            yahoo_ticker
        );
        let resp: YahooChartResponse = self.client.get(&url).send().await?
            .json().await
            .context("Failed to parse Yahoo response for period returns")?;
        let result = resp.chart.result.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("Yahoo returned no result for {}", ticker))?;
        let meta = &result.meta;
        let current_price = meta.regular_market_price;
        let volume = meta.regular_market_volume.unwrap_or(0) as u64;
        let name = meta.long_name.as_ref().or(meta.short_name.as_ref())
            .map(|s| s.clone()).unwrap_or_else(|| ticker.to_string());

        // 일봉 종가 배열 — timestamp와 close를 쌍으로 묶어 None을 건너뛰지 않고
        // 인덱스를 일치시킨 후 유효한 (timestamp, close) 쌍만 추출
        let quote_indicators = result.indicators.quote.into_iter().next().unwrap_or_default();
        let timestamps = result.timestamp;

        // timestamp[i]와 close[i]를 쌍으로 묶어 유효한 것만 추출
        let ts_closes: Vec<(i64, f64)> = timestamps.iter()
            .enumerate()
            .filter_map(|(i, &ts)| {
                let close = quote_indicators.close.get(i).and_then(|c| *c)?;
                Some((ts, close))
            })
            .collect();

        if ts_closes.is_empty() {
            anyhow::bail!("No historical close prices for {}", ticker);
        }

        let now = chrono::Utc::now();
        let periods = vec![
            ("1d", "1일", 1),
            ("1w", "1주", 7),
            ("1m", "1개월", 30),
            ("3m", "3개월", 90),
            ("6m", "6개월", 180),
            ("1y", "1년", 365),
        ];

        let mut returns = Vec::new();
        for (period, label, days) in periods {
            // N일 전의 가격 찾기 (가장 가까운 거래일)
            let target_date = now - chrono::Duration::days(days);
            let mut start_price = ts_closes[0].1;
            for &(ts, close) in &ts_closes {
                let dt = chrono::DateTime::from_timestamp(ts, 0).unwrap_or(now);
                if dt <= target_date {
                    start_price = close;
                } else {
                    break;
                }
            }
            let end_price = current_price;
            let return_pct = if start_price > 0.0 {
                ((end_price - start_price) / start_price) * 100.0
            } else {
                0.0
            };
            returns.push(PeriodReturn {
                period: period.to_string(),
                label: label.to_string(),
                return_pct,
                start_price,
                end_price,
            });
        }

        Ok(PeriodReturns {
            ticker: ticker.to_string(),
            name,
            current_price,
            volume,
            returns,
        })
    }
}

#[derive(Deserialize, Debug)]
struct YahooChartResponse {
    chart: YahooChart,
}

#[derive(Deserialize, Debug)]
struct YahooChart {
    result: Vec<YahooResult>,
    error: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct YahooResult {
    meta: YahooMeta,
    #[serde(default)]
    timestamp: Vec<i64>,
    #[serde(default)]
    indicators: YahooIndicators,
}

#[derive(Deserialize, Debug)]
struct YahooMeta {
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: f64,
    #[serde(rename = "regularMarketVolume")]
    regular_market_volume: Option<i64>,
    #[serde(rename = "chartPreviousClose")]
    chart_previous_close: Option<f64>,
    #[serde(rename = "regularMarketPreviousClose")]
    regular_market_previous_close: Option<f64>,
    #[serde(rename = "regularMarketTime")]
    regular_market_time: Option<i64>,
    #[serde(rename = "shortName")]
    short_name: Option<String>,
    #[serde(rename = "longName")]
    long_name: Option<String>,
    symbol: String,
}

#[derive(Deserialize, Debug, Default)]
struct YahooIndicators {
    #[serde(default)]
    quote: Vec<YahooQuoteIndicators>,
}

#[derive(Deserialize, Debug, Default)]
struct YahooQuoteIndicators {
    #[serde(default, rename = "open")]
    open: Vec<Option<f64>>,
    #[serde(default, rename = "high")]
    high: Vec<Option<f64>>,
    #[serde(default, rename = "low")]
    low: Vec<Option<f64>>,
    #[serde(default, rename = "close")]
    close: Vec<Option<f64>>,
    #[serde(default, rename = "volume")]
    volume: Vec<Option<i64>>,
}

#[async_trait]
impl DataProvider for YahooProvider {
    fn name(&self) -> &str { "Yahoo Finance" }
    fn provider_name(&self) -> ProviderName { ProviderName::Yahoo }

    async fn fetch_quote(&self, ticker: &str) -> Result<NormalizedQuote> {
        let yahoo_ticker = Self::to_yahoo_ticker(ticker);
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=5d",
            yahoo_ticker
        );

        let resp: YahooChartResponse = self.client.get(&url).send().await?
            .json().await
            .context("Failed to parse Yahoo response")?;

        let result = resp.chart.result.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("Yahoo returned no result for {}", ticker))?;

        let meta = &result.meta;
        let current_price = meta.regular_market_price;
        let prev_close = meta.chart_previous_close
            .or(meta.regular_market_previous_close)
            .unwrap_or(0.0);
        let volume = meta.regular_market_volume.unwrap_or(0) as u64;
        let change = current_price - prev_close;
        let change_pct = if prev_close > 0.0 {
            (change / prev_close) * 100.0
        } else {
            0.0
        };
        let name = meta.long_name
            .as_ref()
            .or(meta.short_name.as_ref())
            .map(|s| s.clone())
            .unwrap_or_else(|| ticker.to_string());

        Ok(NormalizedQuote {
            ticker: ticker.to_string(),
            name,
            current_price,
            prev_close,
            change,
            change_pct,
            volume,
            timestamp: Utc::now(),
            provider: ProviderName::Yahoo,
        })
    }

    async fn fetch_batch(&self, tickers: &[&str]) -> Result<Vec<NormalizedQuote>> {
        let mut quotes = Vec::new();
        for ticker in tickers {
            match self.fetch_quote(ticker).await {
                Ok(q) => quotes.push(q),
                Err(e) => tracing::warn!("Yahoo failed for {}: {}", ticker, e),
            }
        }
        Ok(quotes)
    }

    async fn fetch_index(&self, index_code: &str) -> Result<NormalizedQuote> {
        // KOSPI 200 → ^KPI200 또는 코스피 → ^KS11
        let yahoo_code = match index_code {
            "KPI200" | "KOSPI200" => "^KPI200".to_string(),
            "KOSPI" | "KS11" => "^KS11".to_string(),
            "KOSDAQ" => "^KQ11".to_string(),
            other => Self::to_yahoo_ticker(other),
        };
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=5d",
            yahoo_code
        );

        let resp: YahooChartResponse = self.client.get(&url).send().await?
            .json().await?;

        let result = resp.chart.result.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("Yahoo returned no result for index {}", index_code))?;

        let meta = &result.meta;
        let current_price = meta.regular_market_price;
        let prev_close = meta.chart_previous_close
            .or(meta.regular_market_previous_close)
            .unwrap_or(0.0);
        let change = current_price - prev_close;
        let change_pct = if prev_close > 0.0 {
            (change / prev_close) * 100.0
        } else {
            0.0
        };

        Ok(NormalizedQuote {
            ticker: index_code.to_string(),
            name: meta.short_name.clone().unwrap_or_else(|| index_code.to_string()),
            current_price,
            prev_close,
            change,
            change_pct,
            volume: meta.regular_market_volume.unwrap_or(0) as u64,
            timestamp: Utc::now(),
            provider: ProviderName::Yahoo,
        })
    }

    async fn fetch_intraday(&self, ticker: &str) -> Result<Vec<IntradayPoint>> {
        let yahoo_ticker = Self::to_yahoo_ticker(ticker);
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=5m&range=1d",
            yahoo_ticker
        );

        let resp: YahooChartResponse = self.client.get(&url).send().await?
            .json().await?;

        let result = resp.chart.result.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No intraday result for {}", ticker))?;

        let timestamps = result.timestamp;
        let quote_indicators = result.indicators.quote.into_iter().next()
            .unwrap_or_default();
        let closes = quote_indicators.close;
        let volumes = quote_indicators.volume;

        let points = timestamps.iter().enumerate()
            .filter_map(|(i, &ts)| {
                let close = closes.get(i).and_then(|c| *c)?;
                let volume = volumes.get(i).and_then(|v| *v).unwrap_or(0);
                let dt = chrono::DateTime::from_timestamp(ts, 0)?;
                Some(IntradayPoint {
                    timestamp: dt,
                    price: close,
                    volume: volume as u64,
                })
            })
            .collect();

        Ok(points)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.client.get("https://query1.finance.yahoo.com")
            .send().await?.status().is_success())
    }
}