# ETF Tracker — Development Plan

> PRD(ETF_Tracker_PRD_Final1) 및 기술 설계서(TECH_DESIGN.md) 기반 개발 계획
> 구현은 P0(MVP)부터 단계적으로 진행한다.

---

## P0 — MVP (최초 릴리스 필수)

### Phase 1: 프로젝트 초기화 및 기본 인프라

- [ ] **1.1** Tauri v2 + Svelte 프로젝트 스캐폴드 생성
  - `npm create tauri-app` 기반, Svelte + TypeScript 템플릿
  - vite.config.ts, svelte.config.js, tsconfig.json 설정
- [ ] **1.2** Rust 백엔드 의존성 설정 (Cargo.toml)
  - reqwest (async HTTP), rusqlite (SQLite), tokio, serde, tracing
  - tauri-plugin-notification, tauri-plugin-shell (트레이)
- [ ] **1.3** 프로젝트 디렉토리 구조 생성 (TECH_DESIGN.md §9 참조)
- [ ] **1.4** 로깅 시스템 초기 설정
  - tracing + tracing-appender, 14일 rotation, 민감정보 마스킹
- [ ] **1.5** SQLite 연결 및 스키마 마이그레이션 프레임워크
  - schema_version 테이블, 001_init.sql 마이그레이션
  - 실패 시 기존 DB 백업 보존 로직 (REQ-NF-04)

### Phase 2: 데이터 Provider 및 정규화

- [ ] **2.1** ETF 종목 마스터 JSON 구성
  - 국내 ETF 종목(종목명, 단축코드, 시장구분, 상장상태) 수집 및 JSON 작성
  - `src-tauri/data/etf_master.json`
- [ ] **2.2** 시장 캘린더 JSON 구성
  - 2026년 휴장일 포함, `market_calendar.json`
- [ ] **2.3** Provider Trait 및 NormalizedQuote 모델 구현
  - `providers/mod.rs`, `models/mod.rs`
- [ ] **2.4** Naver Finance Provider 구현 (Primary)
  - 현재가, 전일종가, 등락률, 거래량 fetch
  - KOSPI 200 지수 조회
- [ ] **2.5** Yahoo Finance Provider 구현 (Fallback)
  - 동일 필드 정규화, ticker 매핑 (.KS suffix)
- [ ] **2.6** 정규화 로직 구현 (normalizer.rs)
  - 응답 → NormalizedQuote 매핑, 필수 필드 검증
- [ ] **2.7** Fallback 매니저 구현
  - 3회 연속 실패 → 전환, 10분 주기 복구 시도 (REQ-F-19)
  - 필수 필드 2회 연속 누락 → 전환

### Phase 3: Market Calendar 엔진

- [ ] **3.1** MarketState 열거형 및 판별 로직
  - PreOpen / Regular / Closed / Holiday / Unknown
  - 정규장: 09:00~15:30 KST
- [ ] **3.2** 캘린더 JSON 로드 및 휴장일 판별
  - 캘린더 누락 시 Fallback 모드 (기본 정규장 시간 기반)
- [ ] **3.3** Tauri Command `get_market_state` 구현

### Phase 4: Polling Manager

- [ ] **4.1** PollingManager 구조체 및 스케줄러
  - 1/3/5분 간격, 정규장 중에만 자동 폴링
  - 장외/휴장일 중 자동 폴링 중단, 수동 새로고침 허용
- [ ] **4.2** Exponential Backoff 정책
  - 1min → 2min → 4min → 8min → max 30min
- [ ] **4.3** 일시정지/재개 로직 (REQ-F-10)
  - 일시정지 시: 자동 갱신, Provider 복구, 알림 발송 중단
- [ ] **4.4** Tauri Commands: `set_polling_interval`, `pause_polling`, `resume_polling`, `manual_refresh`

### Phase 5: 대시보드 UI (Frontend)

- [ ] **5.1** 기본 레이아웃 및 테마 설정
  - 다크 테마 기본, 반응형 그리드 레이아웃
- [ ] **5.2** 대시보드 최상단: 시장 지수 표시
  - KOSPI 200 현재가, 등락률
  - ETF 당일 등락률 - KOSPI 200 등락률 차이 (REQ-F-03)
- [ ] **5.3** ETF 데이터 그리드 (QuoteGrid.svelte)
  - 컬럼: 종목명, 종목코드, 현재가, 증감액, 등락률, 거래량, 지수대비등락률차이
  - 열 헤더 클릭 정렬 (오름/내림차순) (REQ-F-04)
  - Stale 상태 시각적 표시 (REQ-F-18)
- [ ] **5.4** Tauri invoke 래퍼 (api/tauri.ts)
  - 모든 IPC Commands 타입 안전 래퍼
- [ ] **5.5** Svelte stores 구현
  - quotes, favorites, positions, settings store
- [ ] **5.6** 종목 검색 및 즐겨찾기 UI
  - ETF 마스터 검색 → 추가/제거 (REQ-F-14)
  - SQLite 연동, 재시작 시 복원

### Phase 6: 가상 포지션

- [ ] **6.1** 포지션 입력 UI (PositionPanel.svelte)
  - 매수일, 평균 매수단가, 수량 입력/수정
  - "참고용 가상 포지션 (실제 계좌 미연동)" 명시 (REQ-F-07)
- [ ] **6.2** 평가손익 및 수익률 계산 (REQ-F-08)
  - (현재가 - 매수단가) × 수량, 수익률 %
- [ ] **6.3** SQLite 저장 및 복원
  - virtual_positions 테이블 CRUD
  - 앱 재시작 시 복원 (REQ-F-14)
- [ ] **6.4** 그리드에 평가손익, 수익률 컬럼 추가

### Phase 7: 알림 및 시스템 트레이

- [ ] **7.1** 시스템 트레이 통합 (REQ-F-10)
  - 창 닫기 → 트레이 최소화, 최초 1회 안내 메시지
  - 컨텍스트 메뉴: [열기, 일시정지, 지금 새로고침, 완전 종료]
- [ ] **7.2** 가격 알림 설정 UI (AlertSettings.svelte)
  - 종목별 목표가(상한), 손절가(하한) 입력 (REQ-F-11)
- [ ] **7.3** AlertEngine + Debounce 구현 (REQ-F-12)
  - 임계치 최초 돌파 시 1회 발송
  - 조건 해소(가격 복귀) 후 재돌파 시 재발송
  - 재설정 시 재발송
- [ ] **7.4** 알림 안전 조건 (REQ-F-13)
  - Stale / Provider 장애 / 장 종료 시 알림 중단
- [ ] **7.5** Windows Toast 알림 발송
  - tauri-plugin-notification 연동

### Phase 8: 데이터 내보내기

- [ ] **8.1** CSV 내보내기 구현 (REQ-F-16)
  - UTF-8 BOM 인코딩, KST 시간 표기
  - 파일명: ETF_Export_YYYYMMDD_HHMM.csv
  - 대시보드 그리드 데이터 + 시계열 과거 데이터
- [ ] **8.2** 내보내기 UI (버튼 + 저장 대화상자)

### Phase 9: 데이터 상태 및 Fallback UI

- [ ] **9.1** 데이터 상태 배지 컴포넌트 (StatusBadge.svelte)
  - Live / Stale / Market Closed / Pre-Market / Holiday / Provider Error
- [ ] **9.2** Provider 전환 경고 노출 (REQ-F-19)
  - 대시보드 상단 소스 변경 경고 배너
- [ ] **9.3** 면책 조항 표시 (REQ-NF-05)
  - 앱 실행 시 1회 "본 데이터는 투자 참고용..." 안내

### Phase 10: 통합 테스트 및 빌드

- [ ] **10.1** P0 인수 조건 검증 (PRD §7)
  - 종목 등록/저장, 자동 갱신, Stale, Fallback, 알림 Debounce, 정렬, CSV, 트레이
- [ ] **10.2** 성능 검증 (REQ-NF-01)
  - Idle RAM ≤ 50MB, Polling CPU < 1%, 시작 속도 ≤ 2초
- [ ] **10.3** Windows MSI/NSIS 인스톨러 빌드
  - WebView2 종속성 관리, 코드 서명 (REQ-NF-03)

---

## P1 — 사용성 및 분석 강화 (후속)

- [ ] 장중 스파크라인 (uPlot) (REQ-F-06)
- [ ] 일봉 선형 차트
- [ ] 모멘텀 시그널 (대체 지표, 저유동성 1억 예외) (REQ-F-09)
- [ ] 알림 Snooze 기능
- [ ] 알림 히스토리 로그 뷰어
- [ ] XLSX 내보내기

## P2 — 고급 기능 (후속)

- [ ] 캔들 차트
- [ ] 다중 매수 로트(Lot) 관리
- [ ] 벤치마크 지수 커스텀 설정
- [ ] 외부 캘린더 API 실시간 동기화
- [ ] 증권사 공식 Open API 모듈 (KIS API)

---

## 개발 순서 요약

```
Phase 1 (인프라) → Phase 2 (Provider) → Phase 3 (Calendar)
→ Phase 4 (Polling) → Phase 5 (Dashboard UI)
→ Phase 6 (Position) → Phase 7 (Alert/Tray)
→ Phase 8 (Export) → Phase 9 (Status UI) → Phase 10 (Test/Build)
```

각 Phase 완료 후 `Logs/YYYY-MM-DD-<task>.md`에 기록하고 daily memory에 추가.