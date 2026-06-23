// MarketBar — 대시보드 최상단 시장 상태 바 (REQ-F-03)
// 테마 전환 버튼 + 글씨 크기 조절 버튼 포함

<script lang="ts">
  import { marketState, marketStateLabel, marketStateColor, pollingStatus, pausePollingAction, resumePollingAction, manualRefreshAction, setPollingIntervalAction } from '$lib/stores';
  import { theme, fontScale, setThemeAction, setFontScaleAction, type ThemeName, type FontScale } from '$lib/stores/theme';

  let selectedInterval = $state(1);

  async function onIntervalChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    selectedInterval = parseInt(target.value);
    await setPollingIntervalAction(selectedInterval);
  }

  async function onTogglePause() {
    const ps = $pollingStatus;
    if (ps && ps.paused) {
      await resumePollingAction();
    } else {
      await pausePollingAction();
    }
  }

  async function onManualRefresh() {
    await manualRefreshAction();
  }

  async function onThemeChange(t: ThemeName) {
    await setThemeAction(t);
  }

  async function onFontScaleChange(s: FontScale) {
    await setFontScaleAction(s);
  }

  const themeButtons: { key: ThemeName; label: string }[] = [
    { key: 'dark', label: '🌙' },
    { key: 'light', label: '☀️' },
    { key: 'high-contrast', label: '🌓' },
  ];

  const scaleButtons: { key: FontScale; label: string }[] = [
    { key: 'small', label: 'A-' },
    { key: 'normal', label: 'A' },
    { key: 'large', label: 'A+' },
  ];
</script>

<header class="market-bar">
  <div class="left-section">
    <h1 class="app-title">📊 ETF Tracker</h1>
    <div class="market-state" style="color: {$marketStateColor}">
      <span class="state-dot" style="background: {$marketStateColor}"></span>
      {$marketStateLabel}
    </div>
  </div>

  <div class="right-section">
    {#if $pollingStatus}
      <div class="polling-info">
        {#if $pollingStatus.paused}
          <span class="paused-tag">일시정지</span>
        {:else if $pollingStatus.backoff_level > 0}
          <span class="backoff-tag">백오프 L{$pollingStatus.backoff_level}</span>
        {/if}
        <select value={selectedInterval} onchange={onIntervalChange} class="interval-select">
          <option value={1}>1분</option>
          <option value={3}>3분</option>
          <option value={5}>5분</option>
        </select>
      </div>
    {/if}

    <button class="btn-icon" onclick={onManualRefresh} title="수동 새로고침">
      🔄
    </button>
    <button class="btn-icon" onclick={onTogglePause} title={$pollingStatus && $pollingStatus.paused ? '재개' : '일시정지'}>
      {#if $pollingStatus && $pollingStatus.paused}▶️{:else}⏸️{/if}
    </button>

    <div class="divider"></div>

    <div class="theme-switcher" role="group" aria-label="테마 선택">
      {#each themeButtons as tb (tb.key)}
        <button
          class="theme-btn"
          class:active={$theme === tb.key}
          onclick={() => onThemeChange(tb.key)}
          title="테마: {tb.key}"
          aria-pressed={$theme === tb.key}
        >
          {tb.label}
        </button>
      {/each}
    </div>

    <div class="font-switcher" role="group" aria-label="글씨 크기">
      {#each scaleButtons as sb (sb.key)}
        <button
          class="font-btn"
          class:active={$fontScale === sb.key}
          onclick={() => onFontScaleChange(sb.key)}
          title="글씨 크기: {sb.key}"
          aria-pressed={$fontScale === sb.key}
        >
          {sb.label}
        </button>
      {/each}
    </div>
  </div>
</header>

<style>
  .market-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .left-section { display: flex; align-items: center; gap: 20px; }
  .app-title {
    font-size: calc(1.1rem * var(--font-scale));
    margin: 0; color: var(--text);
  }
  .market-state {
    display: flex; align-items: center; gap: 6px;
    font-size: calc(0.9rem * var(--font-scale)); font-weight: 600;
  }
  .state-dot { width: 8px; height: 8px; border-radius: 50%; }
  .right-section { display: flex; align-items: center; gap: 10px; }
  .polling-info { display: flex; align-items: center; gap: 8px; }
  .paused-tag {
    color: var(--warning);
    font-size: calc(0.8rem * var(--font-scale)); font-weight: 600;
  }
  .backoff-tag {
    color: var(--danger);
    font-size: calc(0.8rem * var(--font-scale)); font-weight: 600;
  }
  .interval-select {
    background: var(--surface-input); color: var(--text);
    border: 1px solid var(--border-strong);
    border-radius: 6px; padding: 4px 8px;
    font-size: calc(0.85rem * var(--font-scale)); cursor: pointer;
  }
  .btn-icon {
    background: var(--surface-3); border: 1px solid var(--border-strong);
    border-radius: 6px; padding: 6px 10px; cursor: pointer;
    font-size: calc(1rem * var(--font-scale)); transition: background 0.2s;
  }
  .btn-icon:hover { background: var(--surface-hover); }
  .divider {
    width: 1px; height: 24px; background: var(--border-strong); margin: 0 4px;
  }
  .theme-switcher, .font-switcher {
    display: flex; gap: 2px;
    background: var(--surface-3); border: 1px solid var(--border-strong);
    border-radius: 6px; padding: 2px;
  }
  .theme-btn, .font-btn {
    background: transparent; border: none; border-radius: 4px;
    padding: 4px 8px; cursor: pointer;
    font-size: calc(0.8rem * var(--font-scale));
    color: var(--text-muted); transition: all 0.15s;
  }
  .theme-btn:hover, .font-btn:hover {
    background: var(--surface-hover); color: var(--text);
  }
  .theme-btn.active, .font-btn.active {
    background: var(--accent); color: #fff;
  }
  .font-btn { font-weight: 600; min-width: 28px; }
</style>