// ProviderBanner — Provider 전환 경고 배너 (REQ-F-19)
// Fallback 모드 진입 시 대시보드 상단에 노란 경고 배너 표시

<script lang="ts">
  import { providerStatus } from '$lib/stores';

  let status = $derived($providerStatus);
</script>

{#if status !== null && status.is_using_fallback}
  <div class="provider-banner">
    <span class="banner-icon">⚠️</span>
    <span class="banner-text">
      데이터 소스가 전환되었습니다. 현재: <strong>{status.active_provider}</strong>
      (Primary 실패 {status.primary_failures}회 — 자동 복구 시도 중)
    </span>
  </div>
{/if}

<style>
  .provider-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--warning-bg);
    border: 1px solid var(--warning-border);
    border-radius: 6px;
    margin-bottom: 12px;
    font-size: calc(0.85rem * var(--font-scale));
    color: var(--warning-text);
  }
  .banner-icon { font-size: calc(1rem * var(--font-scale)); flex-shrink: 0; }
  .banner-text { line-height: 1.4; }
  .banner-text strong { color: var(--warning); }
</style>