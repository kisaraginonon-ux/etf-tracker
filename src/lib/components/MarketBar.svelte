// MarketBar — 대시보드 최상단 시장 상태 바 (REQ-F-03)

<script lang="ts">
  import { marketState, marketStateLabel, marketStateColor, pollingStatus, pausePollingAction, resumePollingAction, manualRefreshAction, setPollingIntervalAction } from '$lib/stores';

  let selectedInterval = $state(1);

  async function onIntervalChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    selectedInterval = parseInt(target.value);
    await setPollingIntervalAction(selectedInterval);
  }

  async function onTogglePause() {
    if ($pollingStatus?.paused) {
      await resumePollingAction();
    } else {
      await pausePollingAction();
    }
  }

  async function onManualRefresh() {
    await manualRefreshAction();
  }
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
    <button class="btn-icon" onclick={onTogglePause} title={$pollingStatus?.paused ? '재개' : '일시정지'}>
      {#if $pollingStatus?.paused}▶️{:else}⏸️{/if}
    </button>
  </div>
</header>

<style>
  .market-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    background: #1e1e2e;
    border-bottom: 1px solid #333;
    flex-shrink: 0;
  }
  .left-section { display: flex; align-items: center; gap: 20px; }
  .app-title { font-size: 1.1rem; margin: 0; color: #e0e0e0; }
  .market-state { display: flex; align-items: center; gap: 6px; font-size: 0.9rem; font-weight: 600; }
  .state-dot { width: 8px; height: 8px; border-radius: 50%; }
  .right-section { display: flex; align-items: center; gap: 10px; }
  .polling-info { display: flex; align-items: center; gap: 8px; }
  .paused-tag { color: #ff9800; font-size: 0.8rem; font-weight: 600; }
  .backoff-tag { color: #f44336; font-size: 0.8rem; font-weight: 600; }
  .interval-select {
    background: #2a2a3a; color: #e0e0e0; border: 1px solid #444;
    border-radius: 6px; padding: 4px 8px; font-size: 0.85rem; cursor: pointer;
  }
  .btn-icon {
    background: #2a2a3a; border: 1px solid #444; border-radius: 6px;
    padding: 6px 10px; cursor: pointer; font-size: 1rem; transition: background 0.2s;
  }
  .btn-icon:hover { background: #3a3a4a; }
</style>