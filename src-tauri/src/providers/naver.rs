// Naver Finance Provider (Primary)
// 네이버 증권 HTML 페이지 스크래핑 기반 시세 데이터 수집
// 엔드포인트: https://finance.naver.com/item/sise.naver?code={ticker}

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::models::{NormalizedQuote, IntradayPoint, ProviderName};
use super::DataProvider;

pub struct NaverProvider {
    client: Client,
}

impl NaverProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    /// HTML에서 숫자 문자열 파싱 (쉼표 제거)
    fn parse_num(s: &str) -> f64 {
        s.replace(',', "")
            .replace("&nbsp;", "")
            .trim()
            .parse()
            .unwrap_or(0.0)
    }

    /// 네이버 증권 페이지에서 시세 데이터 스크래핑
    async fn scrape_quote(&self, ticker: &str) -> Result<ScrapedQuote> {
        let url = format!("https://finance.naver.com/item/sise.naver?code={}", ticker);
        let html = self.client.get(&url).send().await?
            .text().await
            .context("Failed to fetch Naver page")?;

        // 네이버는 euc-kr 인코딩 사용 — reqwest가 자동 변환하지 못할 수 있음
        let html = if html.contains("�") {
            // 인코딩 문제 시 iconv 처리가 필요하지만,
            // reqwest는 보통 charset을 자동 감지함
            html
        } else {
            html
        };

        let document = Html::parse_document(&html);

        // 현재가: id="_nowVal" 내부의 <strong>
        let now_val = Self::extract_text(&document, "strong#_nowVal")
            .or_else(|| Self::extract_text(&document, "span#_nowVal"));

        // 전일가: "전일가" 라벨 옆의 값
        let prev_close = Self::extract_text_by_label(&document, "전일")
            .or_else(|| Self::extract_text(&document, "span.tah.p11"));

        // 전일대비: id="_diff"
        let diff = Self::extract_text(&document, "strong#_diff")
            .or_else(|| Self::extract_text(&document, "span#_diff"));

        // 등락률: id="_rate"
        let rate = Self::extract_text(&document, "strong#_rate")
            .or_else(|| Self::extract_text(&document, "span#_rate"));

        // 거래량: id="_quant"
        let volume = Self::extract_text(&document, "span#_quant");

        // 종목명: 페이지 타이틀 또는 h2
        let name = Self::extract_text(&document, "h2 a")
            .or_else(|| Self::extract_text(&document, "strong a"))
            .unwrap_or_else(|| ticker.to_string());

        Ok(ScrapedQuote {
            name,
            current_price: Self::parse_num(&now_val.unwrap_or_default()),
            prev_close: Self::parse_num(&prev_close.unwrap_or_default()),
            change: Self::parse_num(&diff.unwrap_or_default()),
            change_pct: Self::parse_num(&rate.unwrap_or_default()),
            volume: Self::parse_num(&volume.unwrap_or_default()) as u64,
        })
    }

    /// CSS selector로 첫 번째 매칭 요소의 텍스트 추출
    fn extract_text(doc: &Html, selector_str: &str) -> Option<String> {
        let selector = Selector::parse(selector_str).ok()?;
        doc.select(&selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// 라벨 기반으로 값 추출 (예: "전일" 옆의 숫자)
    fn extract_text_by_label(doc: &Html, label: &str) -> Option<String> {
        let selector = Selector::parse("span.tah.p11").ok()?;
        let spans: Vec<String> = doc.select(&selector)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        // "전일" 라벨 다음 값이 전일가
        // 네이버 시세 페이지 구조상 여러 span.tah.p11이 순서대로 나옴
        // 0: 전일가, 1: 시가, 2: 고가, 3: 저가
        if label == "전일" {
            return spans.first().cloned();
        }
        None
    }
}

struct ScrapedQuote {
    name: String,
    current_price: f64,
    prev_close: f64,
    change: f64,
    change_pct: f64,
    volume: u64,
}

#[async_trait]
impl DataProvider for NaverProvider {
    fn name(&self) -> &str { "Naver Finance" }
    fn provider_name(&self) -> ProviderName { ProviderName::Naver }

    async fn fetch_quote(&self, ticker: &str) -> Result<NormalizedQuote> {
        let scraped = self.scrape_quote(ticker).await?;

        // change/change_pct가 0이면 계산
        let change = if scraped.change != 0.0 {
            scraped.change
        } else {
            scraped.current_price - scraped.prev_close
        };
        let change_pct = if scraped.change_pct != 0.0 {
            scraped.change_pct
        } else if scraped.prev_close > 0.0 {
            (change / scraped.prev_close) * 100.0
        } else {
            0.0
        };

        Ok(NormalizedQuote {
            ticker: ticker.to_string(),
            name: scraped.name,
            current_price: scraped.current_price,
            prev_close: scraped.prev_close,
            change,
            change_pct,
            volume: scraped.volume,
            timestamp: Utc::now(),
            provider: ProviderName::Naver,
        })
    }

    async fn fetch_batch(&self, tickers: &[&str]) -> Result<Vec<NormalizedQuote>> {
        let mut quotes = Vec::new();
        for ticker in tickers {
            match self.fetch_quote(ticker).await {
                Ok(q) => quotes.push(q),
                Err(e) => tracing::warn!("Naver failed for {}: {}", ticker, e),
            }
        }
        Ok(quotes)
    }

    async fn fetch_index(&self, index_code: &str) -> Result<NormalizedQuote> {
        // 코스피 200 등 지수도 동일한 페이지 구조
        // https://finance.naver.com/sise/sise_index.naver?code=KPI200
        let url = format!("https://finance.naver.com/sise/sise_index.naver?code={}", index_code);
        let html = self.client.get(&url).send().await?
            .text().await
            .context("Failed to fetch Naver index page")?;

        let document = Html::parse_document(&html);
        let current = Self::extract_text(&document, "span#now_value")
            .or_else(|| Self::extract_text(&document, "strong#now_value"));
        let change = Self::extract_text(&document, "span#change_value")
            .or_else(|| Self::extract_text(&document, "strong#change_value"));
        let rate = Self::extract_text(&document, "span#change_rate")
            .or_else(|| Self::extract_text(&document, "strong#change_rate"));
        let name = Self::extract_text(&document, "h3.h3_sub")
            .or_else(|| Self::extract_text(&document, ".subjet_top"))
            .unwrap_or_else(|| index_code.to_string());

        Ok(NormalizedQuote {
            ticker: index_code.to_string(),
            name,
            current_price: Self::parse_num(&current.unwrap_or_default()),
            prev_close: 0.0, // 지수는 전일가를 별도로 스크래핑 필요
            change: Self::parse_num(&change.unwrap_or_default()),
            change_pct: Self::parse_num(&rate.unwrap_or_default()),
            volume: 0,
            timestamp: Utc::now(),
            provider: ProviderName::Naver,
        })
    }

    async fn fetch_intraday(&self, _ticker: &str) -> Result<Vec<IntradayPoint>> {
        // TODO: 네이버 장중 시간대별 데이터 (P1)
        Ok(Vec::new())
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.client.get("https://finance.naver.com")
            .send().await?.status().is_success())
    }
}