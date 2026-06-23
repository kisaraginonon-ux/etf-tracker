// Tauri API Wrapper — 모든 IPC 호출의 타입 안전 래퍼
// TECH_DESIGN.md §10 참조

import { invoke } from '@tauri-apps/api/core';
import type {
  MarketState,
  EtfMasterItem,
  EtfListItem,
  Favorite,
  NormalizedQuote,
  PollingStatus,
  ProviderStatus,
  PeriodReturns,
} from '$lib/types';

// === Calendar ===
export async function getMarketState(): Promise<MarketState> {
  return invoke<MarketState>('get_market_state');
}

export async function getMarketProgress(): Promise<number> {
  return invoke<number>('get_market_progress');
}

// === ETF Master ===
export async function searchEtf(query: string): Promise<EtfMasterItem[]> {
  return invoke<EtfMasterItem[]>('search_etf', { query });
}

// === Favorites ===
export async function getFavorites(): Promise<Favorite[]> {
  return invoke<Favorite[]>('get_favorites');
}

export async function addFavorite(ticker: string, name: string, marketSection: string): Promise<void> {
  return invoke<void>('add_favorite', { ticker, name, marketSection });
}

export async function removeFavorite(ticker: string): Promise<void> {
  return invoke<void>('remove_favorite', { ticker });
}

// === Virtual Positions (REQ-F-07, REQ-F-08) ===
import type { VirtualPosition } from '$lib/types';

export async function getPositions(): Promise<VirtualPosition[]> {
  return invoke<VirtualPosition[]>('get_positions');
}

export async function setPosition(
  ticker: string,
  buyDate: string | null,
  avgBuyPrice: number | null,
  quantity: number | null,
): Promise<void> {
  return invoke<void>('set_position', { ticker, buyDate, avgBuyPrice, quantity });
}

export async function removePosition(ticker: string): Promise<void> {
  return invoke<void>('remove_position', { ticker });
}

// === Alerts (REQ-F-11, REQ-F-12) ===
import type { AlertConfig, AlertType } from '$lib/types';

export async function getAlerts(): Promise<AlertConfig[]> {
  return invoke<AlertConfig[]>('get_alerts');
}

export async function setAlert(ticker: string, alertType: AlertType, threshold: number): Promise<void> {
  return invoke<void>('set_alert', { ticker, alertType, threshold });
}

export async function resetAlert(ticker: string): Promise<void> {
  return invoke<void>('reset_alert', { ticker });
}

export async function removeAlert(ticker: string, alertType: AlertType): Promise<void> {
  return invoke<void>('remove_alert', { ticker, alertType });
}

// === Polling ===
export async function setPollingInterval(minutes: number): Promise<boolean> {
  return invoke<boolean>('set_polling_interval', { minutes });
}

export async function pausePolling(): Promise<void> {
  return invoke<void>('pause_polling');
}

export async function resumePolling(): Promise<void> {
  return invoke<void>('resume_polling');
}

export async function getPollingStatus(): Promise<PollingStatus> {
  return invoke<PollingStatus>('get_polling_status');
}

export async function manualRefresh(): Promise<string> {
  return invoke<string>('manual_refresh');
}

// === CSV Export (REQ-F-16) ===
export async function exportCsv(filePath: string): Promise<string> {
  return invoke<string>('export_csv', { filePath });
}

// === Provider Status (REQ-F-19) ===
export async function getProviderStatus(): Promise<ProviderStatus> {
  return invoke<ProviderStatus>('get_provider_status');
}

// === Naver ETF List ===
export async function fetchNaverEtfList(): Promise<EtfListItem[]> {
  return invoke<EtfListItem[]>('fetch_naver_etf_list');
}

// === Manual Quote Fetch (장 마감 후에도 수동 조회) ===
export async function fetchQuoteNow(ticker: string): Promise<NormalizedQuote> {
  return invoke<NormalizedQuote>('fetch_quote_now', { ticker });
}

// === Period Returns (기간별 등락률) ===
export async function fetchPeriodReturns(ticker: string): Promise<PeriodReturns> {
  return invoke<PeriodReturns>('fetch_period_returns', { ticker });
}