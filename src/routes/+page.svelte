// ETF Tracker — Main Page
// 좌측(35%): EtfDetailPanel (선택 종목 상세) | 우측(65%): 탭 (전체 ETF 그리드 | 가격 알림)

<script lang="ts">
  import { onMount } from 'svelte';
  import '$lib/styles/theme.css';
  import MarketBar from '$lib/components/MarketBar.svelte';
  import EtfDetailPanel from '$lib/components/EtfDetailPanel.svelte';
  import EtfGrid from '$lib/components/EtfGrid.svelte';
  import AlertSettings from '$lib/components/AlertSettings.svelte';
  import ProviderBanner from '$lib/components/ProviderBanner.svelte';
  import DisclaimerModal from '$lib/components/DisclaimerModal.svelte';
  import { favorites, loadFavorites, loadPositions, loadAlerts, refreshMarketState, refreshPollingStatus, refreshProviderStatus, loadEtfList, etfList, exportCsvAction } from '$lib/stores';
  import { loadThemeSettings } from '$lib/stores/theme';
  import { save } from '@tauri-apps/plugin-dialog';

  let exportStatus = $state<'idle' | 'exporting' | 'done' | 'error'>('idle');
  let rightTab = $state<'quotes' | 'alerts'>('quotes');

  onMount(() => {
    loadThemeSettings();
    loadFavorites();
    loadPositions();
    loadAlerts();
    refreshMarketState();
    refreshPollingStatus();
    refreshProviderStatus();
    if ($etfList.length === 0) {
      loadEtfList();
    }
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
      <span class="disclaimer">본 데이터는 투자 참고용이며 오류 가능성이 있습니다</span>
      <button class="export-btn" onclick={handleExportCsv} disabled={exportStatus === 'exporting'}>
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

    <div class="split-layout">
      <!-- 좌측 (35%): 선택 종목 상세 패널 -->
      <section class="left-pane">
        <EtfDetailPanel />
      </section>

      <!-- 우측 (65%): 전체 ETF 그리드 / 가격 알림 (탭) -->
      <section class="right-pane">
        <div class="tab-bar">
          <button class="tab-btn" class:active={rightTab === 'quotes'} onclick={() => rightTab = 'quotes'}>
            📈 전체 ETF 그리드
          </button>
          <button class="tab-btn" class:active={rightTab === 'alerts'} onclick={() => rightTab = 'alerts'}>
            🔔 가격 알림
          </button>
        </div>
        <div class="tab-content">
          {#if rightTab === 'quotes'}
            <EtfGrid />
          {:else}
            <AlertSettings />
          {/if}
        </div>
      </section>
    </div>
  </main>
</div>

<DisclaimerModal />

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: var(--font-family);
    background: var(--bg);
    color: var(--text);
  }
  :global(*) { box-sizing: border-box; }
  :global(::-webkit-scrollbar) { width: 8px; height: 8px; }
  :global(::-webkit-scrollbar-track) { background: var(--scrollbar-track); }
  :global(::-webkit-scrollbar-thumb) { background: var(--scrollbar-thumb); border-radius: 4px; }
  :global(::-webkit-scrollbar-thumb:hover) { background: var(--scrollbar-thumb-hover); }

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
    padding: 12px 16px;
    overflow: hidden;
    font-size: calc(0.95rem * var(--font-scale));
    gap: 8px;
  }
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 4px;
  }
  .disclaimer {
    font-size: calc(0.75rem * var(--font-scale));
    color: var(--text-dim);
  }
  .export-btn {
    background: var(--accent-bg);
    color: var(--accent);
    border: 1px solid var(--accent-border);
    border-radius: 6px;
    padding: 6px 14px;
    cursor: pointer;
    font-size: calc(0.85rem * var(--font-scale));
    font-weight: 500;
  }
  .export-btn:hover:not(:disabled) {
    background: var(--accent-bg-hover);
  }
  .export-btn:disabled {
    opacity: 0.5;
    cursor: wait;
  }
  .split-layout {
    flex: 1;
    display: flex;
    gap: 12px;
    overflow: hidden;
  }
  .left-pane {
    width: 35%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding-right: 4px;
  }
  .right-pane {
    width: 65%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .tab-bar {
    display: flex;
    gap: 4px;
    margin-bottom: 8px;
    border-bottom: 1px solid var(--border);
    padding-bottom: 4px;
  }
  .tab-btn {
    background: var(--surface-3);
    color: var(--text-muted);
    border: 1px solid var(--border);
    border-radius: 6px 6px 0 0;
    padding: 6px 16px;
    cursor: pointer;
    font-size: calc(0.88rem * var(--font-scale));
    font-weight: 500;
  }
  .tab-btn:hover {
    background: var(--surface-hover);
  }
  .tab-btn.active {
    background: var(--accent);
    color: var(--text);
    border-color: var(--accent-border);
  }
  .tab-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>