// StatusBadge — 데이터 상태 배지 컴포넌트 (REQ-F-18)

<script lang="ts">
  import type { DataStatus } from '$lib/types';

  let { status }: { status: DataStatus } = $props();

  const badgeConfig: Record<DataStatus, { label: string; color: string; bg: string }> = {
    live:           { label: 'Live',           color: '#4caf50', bg: 'rgba(76,175,80,0.15)' },
    stale:          { label: 'Stale',           color: '#ff9800', bg: 'rgba(255,152,0,0.15)' },
    market_closed:  { label: '장마감',          color: '#888',    bg: 'rgba(136,136,136,0.15)' },
    pre_market:     { label: '장전',            color: '#666',    bg: 'rgba(102,102,102,0.15)' },
    holiday:        { label: '시장 미운영',     color: '#f44336', bg: 'rgba(244,67,54,0.15)' },
    provider_error: { label: '데이터 오류',    color: '#f44336', bg: 'rgba(244,67,54,0.15)' },
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
    font-size: 0.75rem;
    font-weight: 600;
    white-space: nowrap;
  }
</style>