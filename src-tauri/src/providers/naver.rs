// Naver Finance Provider (Primary)
// 네이버 증권 HTML 페이지 스크래핑 기반 시세 데이터 수집
// 엔드포인트: https://finance.naver.com/item/sise.naver?code={ticker}

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use regex::Regex;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};

use crate::models::{NormalizedQuote, IntradayPoint, ProviderName, EtfListItem};
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
        let html = if html.contains("") {
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

    /// 숫자 문자열에서 부호를 분리 — ("+1,234", "0.50") → f64 (부호 적용)
    /// 네이버 목록 테이블의 등락률/등락액 컬럼은 부호가 붙은 문자열
    fn parse_signed_num(s: &str) -> f64 {
        let trimmed = s.replace(',', "").replace("&nbsp;", "").trim().to_string();
        // 부호 판별 후 절대값 파싱
        let (sign, rest) = if let Some(stripped) = trimmed.strip_prefix('-') {
            (-1.0, stripped)
        } else if let Some(stripped) = trimmed.strip_prefix('+') {
            (1.0, stripped)
        } else {
            (1.0, trimmed.as_str())
        };
        let val: f64 = rest.parse().unwrap_or(0.0);
        sign * val
    }

    /// 6자리 숫자 종목코드 추출 (문자열에서 첫 6자리 연속 숫자)
    fn extract_ticker(s: &str) -> Option<String> {
        let re = Regex::new(r"\b(\d{6})\b").ok()?;
        re.captures(s)
            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
    }

    /// 네이버 전체 ETF 목록 스크래핑 (sise_market_sum.naver?menu=etf)
    /// 여러 페이지를 순회하며 종목코드/종목명/현재가/등락률/거래량 수집
    pub async fn fetch_etf_list(&self) -> Result<Vec<EtfListItem>> {
        let mut items: Vec<EtfListItem> = Vec::new();
        // 1~10페이지 순회 (페이지당 ~50개)
        for page in 1..=10u32 {
            match self.scrape_etf_list_page(page).await {
                Ok(page_items) => {
                    let empty = page_items.is_empty();
                    items.extend(page_items);
                    if empty {
                        // 더 이상 데이터 없으면 중단
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Naver ETF list page {} failed: {}", page, e);
                    // 페이지 실패 시 부분 결과라도 반환하며 중단
                    break;
                }
            }
        }
        Ok(items)
    }

    /// 단일 ETF 목록 페이지 스크래핑
    async fn scrape_etf_list_page(&self, page: u32) -> Result<Vec<EtfListItem>> {
        let url = format!(
            "https://finance.naver.com/sise/sise_market_sum.naver?&menu=etf&sosok=0&page={}",
            page
        );
        let html = self
            .client
            .get(&url)
            .send()
            .await?
            .text()
            .await
            .with_context(|| format!("Failed to fetch Naver ETF list page {}", page))?;

        let document = Html::parse_document(&html);
        let mut items = Vec::new();

        // 네이버 시세종액 테이블: table.type_2 > tr (또는 .type_2)
        // 헤더 행은 건너뛰고, td가 2개 미만인 빈 행도 건너뜀
        let tr_selector = Selector::parse("table.type_2 tr")
            .or_else(|_| Selector::parse(".type_2 tr"))
            .map_err(|e| anyhow::anyhow!("Selector parse error: {}", e))?;
        let td_selector = Selector::parse("td").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        for tr in document.select(&tr_selector) {
            let tds: Vec<ElementRef> = tr.select(&td_selector).collect();
            if tds.len() < 2 {
                continue; // 헤더/빈 행
            }

            // 컬럼 순서 (네이버 시세종액 ETF): 종목명, 현재가, 전일비, 등락률, 거래량, 거래대금, ...
            // 종목명 셀 내에 <a href="/item/main.naver?code=XXXXXX">종목명</a> 형태
            let name_cell = &tds[0];
            let name = name_cell.text().collect::<String>().trim().to_string();
            if name.is_empty() {
                continue;
            }

            // 종목코드: <a>의 href 속성에서 code=XXXXXX 추출
            let ticker = match name_cell.select(&a_selector).next() {
                Some(a) => {
                    let href = a.value().attr("href").unwrap_or("");
                    Self::extract_ticker(href).unwrap_or_default()
                }
                None => String::new(),
            };
            if ticker.len() != 6 {
                continue; // 6자리 코드가 아니면 ETF가 아닐 가능성 — 스킵
            }

            // 현재가 (컬럼 인덱스 1)
            let current_price = Self::parse_num(
                &tds.get(1).map(|c| c.text().collect::<String>()).unwrap_or_default(),
            );

            // 등락률 (컬럼 인덱스 3 — 전일비 2, 등락률 3)
            let change_pct = tds
                .get(3)
                .map(|c| {
                    let txt = c.text().collect::<String>();
                    Self::parse_signed_num(&txt)
                })
                .unwrap_or(0.0);

            // 거래량 (컬럼 인덱스 4)
            let volume = tds
                .get(4)
                .map(|c| Self::parse_num(&c.text().collect::<String>()) as u64)
                .unwrap_or(0);

            items.push(EtfListItem {
                ticker,
                name,
                current_price,
                change_pct,
                volume,
            });
        }

        Ok(items)
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