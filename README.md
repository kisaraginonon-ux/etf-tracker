# ETF Tracker

Windows 데스크톱 ETF 데이터 트래커 — 초개인 투자자를 위한 경량 모니터링 보조 도구.

## 문서
- **PRD**: `ETF_Tracker_PRD_Final1.pdf` — 제품 요구사항 정의서
- **TECH_DESIGN.md** — 기술 설계서 (아키텍처, 모듈, DB 스키마, API 정규화)
- **Plan.md** — 개발 계획 (P0/P1/P2 단계별)

## 기술 스택
- **Framework**: Tauri v2 (Rust + WebView2)
- **Frontend**: Svelte 5 + TypeScript
- **Backend**: Rust (reqwest, rusqlite, tokio, tracing)
- **DB**: SQLite (rusqlite, WAL 모드)
- **Installer**: NSIS / MSI (Tauri 번들)
- **Target**: Windows 10/11

## P0 기능 (MVP)
- ✅ ETF 종목 검색 및 즐겨찾기 등록/저장 (REQ-F-14)
- ✅ 자동 갱신 폴링 (1/3/5분, 백오프, 정규장 중만) (REQ-F-05)
- ✅ Market Calendar (KST, 휴장일, 5단계 MarketState) (REQ-F-02)
- ✅ 다크 테마 대시보드 (MarketBar, QuoteGrid, 정렬) (REQ-F-01/03/04)
- ✅ 가상 포지션 (매수일/단가/수량, 평가손익) (REQ-F-07/08)
- ✅ 가격 알림 (목표가/손절가, Debounce, 재돌파 재발송) (REQ-F-11/12/13)
- ✅ 시스템 트레이 (최소화, 컨텍스트 메뉴) (REQ-F-10)
- ✅ CSV 내보내기 (UTF-8 BOM, KST, 저장 대화상자) (REQ-F-16)
- ✅ 데이터 상태 배지 (Live/Stale/Closed/PreMarket/Holiday/Error) (REQ-F-18)
- ✅ Provider Fallback (Naver→Yahoo, 3회 실패 전환, 10분 복구) (REQ-F-19)
- ✅ Provider 전환 경고 배너 (REQ-F-19)
- ✅ 면책 조항 모달 (최초 1회) (REQ-NF-05)
- ✅ 로깅 (tracing, 14일 rotation, 민감정보 마스킹) (REQ-NF-02)
- ✅ DB 백업 (마이그레이션 실패 시 복구) (REQ-NF-04)

## 통계
- **Rust**: 17 모듈, ~2,189 LOC
- **Svelte**: 8 컴포넌트, ~809 LOC
- **TypeScript**: 3 파일, ~450 LOC
- **IPC Commands**: 22개
- **단위 테스트**: 22개 (calendar 7 + polling 7 + alert 8)
- **프론트엔드 번들**: ~204KB
- **릴리즈 바이너리**: ~25MB

## 개발

```bash
# 개발 모드
npm run dev

# 프로덕션 빌드
npm run build

# 인스톨러 빌드 (Windows)
npm run tauri build
```

## 상태
✅ P0 MVP 구현 완료 (Phase 1~10)