// Types — 프론트엔드 타입 정의 (백엔드 models와 대응)

export type MarketState = 'pre_open' | 'regular' | 'closed' | 'holiday' | 'unknown';

export type ProviderName = 'naver' | 'yahoo';

export type DataStatus = 'live' | 'stale' | 'market_closed' | 'pre_market' | 'holiday' | 'provider_error';

export type AlertType = 'target' | 'stop_loss';

export type SignalState = 'provisional' | 'confirmed';

export interface NormalizedQuote {
  ticker: string;
  name: string;
  current_price: number;
  prev_close: number;
  change: number;
  change_pct: number;
  volume: number;
  timestamp: string;
  provider: ProviderName;
}

export interface EtfMasterItem {
  ticker: string;
  name: string;
  market_section: string;
  is_active: boolean;
}

export interface Favorite {
  ticker: string;
  name: string;
  market_section: string;
  added_at: string;
  is_active: boolean;
}

export interface VirtualPosition {
  ticker: string;
  buy_date: string | null;
  avg_buy_price: number | null;
  quantity: number | null;
}

export interface AlertConfig {
  ticker: string;
  alert_type: AlertType;
  threshold: number;
  is_active: boolean;
}

export interface PollingStatus {
  paused: boolean;
  interval_minutes: number;
  backoff_level: number;
  current_delay_secs: number;
  can_auto_poll: boolean;
  market_state: MarketState;
}

export interface ProviderStatus {
  active_provider: string;
  is_using_fallback: boolean;
  primary_failures: number;
}

export interface MarketIndex {
  name: string;
  code: string;
  current_price: number;
  change: number;
  change_pct: number;
}

export interface EtfListItem {
  ticker: string;
  name: string;
  current_price: number;
  change_pct: number;
  volume: number;
}

export interface PeriodReturn {
  period: string;
  label: string;
  return_pct: number;
  start_price: number;
  end_price: number;
}

export interface PeriodReturns {
  ticker: string;
  name: string;
  current_price: number;
  volume: number;
  returns: PeriodReturn[];
}

export interface QuoteRow {
  ticker: string;
  name: string;
  current_price: number;
  change: number;
  change_pct: number;
  volume: number;
  index_diff: number; // ETF 등락률 - KOSPI200등락률
  status: DataStatus;
  position?: VirtualPosition | null;
  eval_profit?: number | null;
  eval_profit_pct?: number | null;
}