// Stores — Svelte 반응형 상태 관리

import { writable, derived } from 'svelte/store';
import type {
  Favorite,
  EtfMasterItem,
  PollingStatus,
  MarketState,
} from '$lib/types';
import * as api from '$lib/api/tauri';

// === Favorites ===
export const favorites = writable<Favorite[]>([]);
export const favoritesLoading = writable(false);

// === Virtual Positions (REQ-F-07) ===
import type { VirtualPosition } from '$lib/types';

export const positions = writable<VirtualPosition[]>([]);

export async function loadPositions() {
  try {
    const result = await api.getPositions();
    positions.set(result);
  } catch (e) {
    console.error('Failed to load positions:', e);
  }
}

export async function setPositionAction(ticker: string, buyDate: string | null, avgPrice: number | null, qty: number | null) {
  try {
    await api.setPosition(ticker, buyDate, avgPrice, qty);
    await loadPositions();
  } catch (e) {
    console.error('Failed to set position:', e);
  }
}

export async function removePositionAction(ticker: string) {
  try {
    await api.removePosition(ticker);
    await loadPositions();
  } catch (e) {
    console.error('Failed to remove position:', e);
  }
}

// === Derived: position map (ticker → position) ===
export const positionMap = derived(positions, ($positions) => {
  const map = new Map<string, VirtualPosition>();
  for (const p of $positions) {
    map.set(p.ticker, p);
  }
  return map;
});

// === Alerts (REQ-F-11) ===
import type { AlertConfig, AlertType } from '$lib/types';

export const alerts = writable<AlertConfig[]>([]);

export async function loadAlerts() {
  try {
    const result = await api.getAlerts();
    alerts.set(result);
  } catch (e) {
    console.error('Failed to load alerts:', e);
  }
}

export async function setAlertAction(ticker: string, alertType: AlertType, threshold: number) {
  try {
    await api.setAlert(ticker, alertType, threshold);
    await loadAlerts();
  } catch (e) {
    console.error('Failed to set alert:', e);
  }
}

export async function resetAlertAction(ticker: string) {
  try {
    await api.resetAlert(ticker);
    await loadAlerts();
  } catch (e) {
    console.error('Failed to reset alert:', e);
  }
}

export async function removeAlertAction(ticker: string, alertType: AlertType) {
  try {
    await api.removeAlert(ticker, alertType);
    await loadAlerts();
  } catch (e) {
    console.error('Failed to remove alert:', e);
  }
}

export async function loadFavorites() {
  favoritesLoading.set(true);
  try {
    const result = await api.getFavorites();
    favorites.set(result);
  } catch (e) {
    console.error('Failed to load favorites:', e);
  } finally {
    favoritesLoading.set(false);
  }
}

export async function addFavoriteAction(item: EtfMasterItem) {
  try {
    await api.addFavorite(item.ticker, item.name, item.market_section);
    await loadFavorites();
  } catch (e) {
    console.error('Failed to add favorite:', e);
  }
}

export async function removeFavoriteAction(ticker: string) {
  try {
    await api.removeFavorite(ticker);
    await loadFavorites();
  } catch (e) {
    console.error('Failed to remove favorite:', e);
  }
}

// === ETF Search ===
export const searchQuery = writable('');
export const searchResults = writable<EtfMasterItem[]>([]);
export const searching = writable(false);

export async function performSearch(query: string) {
  if (!query.trim()) {
    searchResults.set([]);
    return;
  }
  searching.set(true);
  try {
    const results = await api.searchEtf(query);
    searchResults.set(results);
  } catch (e) {
    console.error('Search failed:', e);
    searchResults.set([]);
  } finally {
    searching.set(false);
  }
}

// === Market State ===
export const marketState = writable<MarketState>('unknown');

export async function refreshMarketState() {
  try {
    const state = await api.getMarketState();
    marketState.set(state);
  } catch (e) {
    console.error('Failed to get market state:', e);
  }
}

// === Polling ===
export const pollingStatus = writable<PollingStatus | null>(null);

export async function refreshPollingStatus() {
  try {
    const status = await api.getPollingStatus();
    pollingStatus.set(status);
  } catch (e) {
    console.error('Failed to get polling status:', e);
  }
}

export async function setPollingIntervalAction(minutes: number) {
  try {
    await api.setPollingInterval(minutes);
    await refreshPollingStatus();
  } catch (e) {
    console.error('Failed to set interval:', e);
  }
}

export async function pausePollingAction() {
  try {
    await api.pausePolling();
    await refreshPollingStatus();
  } catch (e) {
    console.error('Failed to pause:', e);
  }
}

export async function resumePollingAction() {
  try {
    await api.resumePolling();
    await refreshPollingStatus();
  } catch (e) {
    console.error('Failed to resume:', e);
  }
}

export async function manualRefreshAction() {
  try {
    await api.manualRefresh();
  } catch (e) {
    console.error('Manual refresh failed:', e);
  }
}

// === CSV Export (REQ-F-16) ===
export async function exportCsvAction(filePath: string): Promise<string | null> {
  try {
    const result = await api.exportCsv(filePath);
    return result;
  } catch (e) {
    console.error('CSV export failed:', e);
    return null;
  }
}

// === Provider Status (REQ-F-19) ===
import type { ProviderStatus } from '$lib/types';

export const providerStatus = writable<ProviderStatus | null>(null);

export async function refreshProviderStatus() {
  try {
    const status = await api.getProviderStatus();
    providerStatus.set(status);
  } catch (e) {
    console.error('Failed to get provider status:', e);
  }
}

// === Derived: market state label (Korean) ===
export const marketStateLabel = derived(marketState, ($state) => {
  const labels: Record<MarketState, string> = {
    pre_open: '장전',
    regular: '정규장',
    closed: '장마감',
    holiday: '시장 미운영',
    unknown: '알 수 없음',
  };
  return labels[$state] || $state;
});

// === Derived: market state color ===
export const marketStateColor = derived(marketState, ($state) => {
  const colors: Record<MarketState, string> = {
    pre_open: '#888',
    regular: '#4caf50',
    closed: '#666',
    holiday: '#f44336',
    unknown: '#888',
  };
  return colors[$state] || '#888';
});