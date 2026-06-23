// ETF Tracker — Main Page
// 다크 테마 대시보드: MarketBar + SearchPanel + PositionPanel + AlertSettings + QuoteGrid

<script lang="ts">
  import { onMount } from 'svelte';
  import MarketBar from '$lib/components/MarketBar.svelte';
  import SearchPanel from '$lib/components/SearchPanel.svelte';
  import PositionPanel from '$lib/components/PositionPanel.svelte';
  import AlertSettings from '$lib/components/AlertSettings.svelte';
  import QuoteGrid from '$lib/components/QuoteGrid.svelte';
  import ProviderBanner from '$lib/components/ProviderBanner.svelte';
  import DisclaimerModal from '$lib/components/DisclaimerModal.svelte';
  import { favorites, loadFavorites, loadPositions, loadAlerts, refreshMarketState, refreshPollingStatus, refreshProviderStatus, exportCsvAction } from '$lib/stores';
  import { save } from '@tauri-apps/plugin-dialog';

  let showSearch = $state(true);
  let showPositions = $state(false);
  let showAlerts = $state(false);
  let exportStatus = $state<'idle' | 'exporting' | 'done' | 'error'>('idle');

  onMount(() => {
    loadFavorites();
    loadPositions();
    loadAlerts();
    refreshMarketState();
    refreshPollingStatus();
    refreshProviderStatus();
  });

  async function handleExportCsv() {
    exportStatus = 'exporting';
    try {
      const now = new Date();
      const ymd = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}`;
      const hm = `${String(now.getHours()).padStart(2, '0')}${String(now.getMinutes()).padStart(2, '0')}`;
      const defaultName = `ETF_Export_${ymd}_${hm}.csv`;

      const filePath = await save({
        defaultPath: defaultName,
        filters: [{ name: 'CSV', extensions: ['csv'] }],
      });

      if (filePath === null) {
        exportStatus = 'idle';
        return;
      }

      const result = await exportCsvAction(filePath);
      if (result !== null) {
        exportStatus = 'done';
        // 파일이 저장된 폴더 열기
        // (선택적: 저장된 파일 경로 표시)
        setTimeout(() => { exportStatus = 'idle'; }, 3000);
      } else {
        exportStatus = 'error';
        setTimeout(() => { exportStatus = 'idle'; }, 3000);
      }
    } catch (e) {
      console.error('Export failed:', e);
      exportStatus = 'error';
      setTimeout(() => { exportStatus = 'idle'; }, 3000);
    }
  }
</script>

<div class="app">
  <MarketBar />

  <main class="main-content">
    <ProviderBanner />
    <div class="toolbar">
      <div class="toolbar-left">
        <button class="toggle-btn" onclick={() => showSearch = !showSearch}>
          {showSearch ? '🔍 검색 숨기기' : '🔍 종목 검색'}
        </button>
        <button class="toggle-btn" onclick={() => showPositions = !showPositions}>
          {showPositions ? '📊 포지션 숨기기' : '📊 가상 포지션'}
        </button>
        <button class="toggle-btn" onclick={() => showAlerts = !showAlerts}>
          {showAlerts ? '🔔 알림 숨기기' : '🔔 가격 알림'}
        </button>
        <button class="toggle-btn export-btn" onclick={handleExportCsv} disabled={exportStatus === 'exporting'}>
          {#if exportStatus === 'exporting'}
            ⏳ 내보내는 중...
          {:else if exportStatus === 'done'}
            ✅ 내보내기 완료!
          {:else if exportStatus === 'error'}
            ❌ 내보내기 실패
          {:else}
            📥 CSV 내보내기
          {/if}
        </button>
      </div>
      <span class="disclaimer">본 데이터는 투자 참고용이며 오류 가능성이 있습니다</span>
    </div>

    {#if showSearch}
      <SearchPanel />
    {/if}

    {#if showPositions}
      <PositionPanel />
    {/if}

    {#if showAlerts}
      <AlertSettings />
    {/if}

    <QuoteGrid favorites={$favorites} />
  </main>
</div>

<DisclaimerModal />

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: 'Inter', 'Noto Sans KR', -apple-system, BlinkMacSystemFont, sans-serif;
    background: #15151e;
    color: #e0e0e0;
  }
  :global(*) { box-sizing: border-box; }
  :global(::-webkit-scrollbar) { width: 8px; height: 8px; }
  :global(::-webkit-scrollbar-track) { background: #1e1e2e; }
  :global(::-webkit-scrollbar-thumb) { background: #333; border-radius: 4px; }
  :global(::-webkit-scrollbar-thumb:hover) { background: #444; }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 16px 20px;
    overflow: hidden;
  }
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }
  .toolbar-left { display: flex; gap: 8px; flex-wrap: wrap; }
  .toggle-btn {
    background: #2a2a3a; color: #6366f1; border: 1px solid #333;
    border-radius: 6px; padding: 6px 14px; cursor: pointer;
    font-size: 0.85rem; font-weight: 500;
  }
  .toggle-btn:hover { background: #3a3a4a; }
  .export-btn { border-color: #6366f1; color: #818cf8; }
  .export-btn:hover { background: #31314a; }
  .export-btn:disabled { opacity: 0.5; cursor: wait; }
  .disclaimer { font-size: 0.75rem; color: #666; }
</style>