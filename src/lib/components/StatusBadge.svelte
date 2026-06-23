// StatusBadge — 데이터 상태 배지 컴포넌트 (REQ-F-18)

<script lang="ts">
  import type { DataStatus } from '$lib/types';

  let { status }: { status: DataStatus } = $props();

  // 색상은 CSS 변수에서 가져옴 — 의미 색상은 테마 무관
  const badgeConfig: Record<DataStatus, { label: string; color: string; bg: string }> = {
    live:           { label: 'Live',           color: 'var(--status-live)',   bg: 'rgba(76,175,80,0.15)' },
    stale:          { label: 'Stale',          color: 'var(--status-stale)',  bg: 'rgba(255,152,0,0.15)' },
    market_closed:  { label: '장마감',         color: 'var(--status-closed)', bg: 'rgba(136,136,136,0.15)' },
    pre_market:     { label: '장전',           color: 'var(--status-pre)',    bg: 'rgba(102,102,102,0.15)' },
    holiday:        { label: '시장 미운영',    color: 'var(--status-error)',  bg: 'rgba(244,67,54,0.15)' },
    provider_error: { label: '데이터 오류',    color: 'var(--status-error)',  bg: 'rgba(244,67,54,0.15)' },
  };

  let config = $derived(badgeConfig[status] || badgeConfig.live);
</script>

<span class="badge" style="color: {config.color}; background: {config.bg}">
  {config.label}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: 10px;
    font-size: calc(0.75rem * var(--font-scale));
    font-weight: 600;
    white-space: nowrap;
  }
</style>