// EtfListPanel — 네이버 전체 ETF 목록 패널 (SearchPanel 대체)
// onMount 시 자동 스크래핑, 즉시 필터, 행 클릭으로 즐겨찾기 토글

<script lang="ts">
  import { onMount } from 'svelte';
  import {
    etfList,
    etfListError,
    etfListLoading,
    loadEtfList,
    favorites,
    addFavoriteAction,
    removeFavoriteAction,
  } from '$lib/stores';
  import type { EtfListItem, Favorite } from '$lib/types';

  let filterText = $state('');

  // 즐겨찾기 ticker 집합 (빠른 조회)
  let favoriteTickers = $derived(new Set($favorites.map((f: Favorite) => f.ticker)));

  // 필터링된 목록
  let filteredList = $derived(
    filterText.trim() === ''
      ? $etfList
      : $etfList.filter((item: EtfListItem) => {
          const q = filterText.trim().toLowerCase();
          return (
            item.name.toLowerCase().indexOf(q) !== -1 ||
            item.ticker.toLowerCase().indexOf(q) !== -1
          );
        })
  );

  onMount(() => {
    if ($etfList.length === 0) {
      loadEtfList();
    }
  });

  async function onRefresh() {
    await loadEtfList();
  }

  async function onToggleFavorite(item: EtfListItem) {
    if (favoriteTickers.has(item.ticker)) {
      await removeFavoriteAction(item.ticker);
    } else {
      // EtfListItem → Favorite 변환 (market_section은 ETF 목록에 없으므로 빈 문자열)
      await addFavoriteAction({
        ticker: item.ticker,
        name: item.name,
        market_section: '',
        is_active: true,
        added_at: '',
      } as Favorite);
    }
  }

  function formatPrice(n: number): string {
    if (n === 0) return '-';
    return n.toLocaleString('ko-KR');
  }

  function formatPct(n: number): string {
    if (n === 0) return '0.00%';
    return (n >= 0 ? '+' : '') + n.toFixed(2) + '%';
  }

  function colorForChange(n: number): string {
    if (n > 0) return 'var(--color-up)';
    if (n < 0) return 'var(--color-down)';
    return 'var(--color-flat)';
  }
</script>

<div class="etf-list-panel">
  <div class="panel-header">
    <h3>📋 ETF 전체 목록</h3>
    <button class="refresh-btn" onclick={onRefresh} disabled={$etfListLoading}>
      {#if $etfListLoading}불러오는 중...{:else}목록 새로고침{/if}
    </button>
  </div>

  <input
    type="text"
    class="filter-input"
    placeholder="종목명 또는 코드로 필터..."
    bind:value={filterText}
  />

  {#if $etfListLoading && $etfList.length === 0}
    <div class="loading-state">
      <p>⏳ ETF 목록을 불러오는 중...</p>
    </div>
  {:else if $etfListError}
    <div class="error-state">
      <p>ETF 목록을 불러오지 못했습니다.</p>
      <p class="error-message">{$etfListError}</p>
    </div>
  {:else if filteredList.length === 0}
    <div class="empty-state">
      <p>표시할 ETF가 없습니다.</p>
      {#if $etfList.length === 0}
        <p class="hint">"목록 새로고침" 버튼을 눌러 불러오세요.</p>
      {/if}
    </div>
  {:else}
    <div class="list-scroll">
      <table class="etf-table">
        <thead>
          <tr>
            <th class="star-col">★</th>
            <th class="name-col">종목명</th>
            <th class="code-col">코드</th>
            <th class="price-col">현재가</th>
            <th class="pct-col">등락률</th>
          </tr>
        </thead>
        <tbody>
          {#each filteredList as item (item.ticker)}
            <tr
              class="etf-row"
              class:is-favorite={favoriteTickers.has(item.ticker)}
              onclick={() => onToggleFavorite(item)}
              title="클릭하여 즐겨찾기 추가/제거"
            >
              <td class="star-cell">
                <span class="star">{favoriteTickers.has(item.ticker) ? '★' : '☆'}</span>
              </td>
              <td class="name-cell">{item.name}</td>
              <td class="code-cell">{item.ticker}</td>
              <td class="price-cell">{formatPrice(item.current_price)}</td>
              <td class="pct-cell" style="color: {colorForChange(item.change_pct)}">
                {formatPct(item.change_pct)}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  <div class="panel-footer">
    <span class="count-info">총 {filteredList.length}개</span>
    <span class="fav-info">즐겨찾기 {$favorites.length}개</span>
  </div>
</div>

<style>
  .etf-list-panel {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .panel-header h3 {
    margin: 0;
    font-size: calc(1rem * var(--font-scale));
    color: var(--text);
  }
  .refresh-btn {
    background: var(--accent-bg);
    color: var(--accent);
    border: 1px solid var(--accent-border);
    border-radius: 6px;
    padding: 4px 12px;
    cursor: pointer;
    font-size: calc(0.8rem * var(--font-scale));
  }
  .refresh-btn:hover:not(:disabled) {
    background: var(--accent-bg-hover);
  }
  .refresh-btn:disabled {
    opacity: 0.6;
    cursor: wait;
  }
  .filter-input {
    width: 100%;
    background: var(--surface-input);
    color: var(--text);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    padding: 8px 10px;
    font-size: calc(0.88rem * var(--font-scale));
  }
  .filter-input:focus {
    border-color: var(--accent);
    outline: none;
  }
  .loading-state,
  .empty-state,
  .error-state {
    text-align: center;
    padding: 24px 12px;
    color: var(--text-dim);
    font-size: calc(0.88rem * var(--font-scale));
  }
  .error-state {
    color: var(--danger);
  }
  .error-message {
    color: var(--text-muted);
    font-size: calc(0.78rem * var(--font-scale));
    margin-top: 6px;
    overflow-wrap: anywhere;
  }
  .empty-state .hint {
    font-size: calc(0.8rem * var(--font-scale));
    margin-top: 6px;
    color: var(--text-muted);
  }
  .list-scroll {
    max-height: 300px;
    overflow-y: auto;
    border: 1px solid var(--border-soft);
    border-radius: 6px;
  }
  .etf-table {
    width: 100%;
    border-collapse: collapse;
    font-size: calc(0.85rem * var(--font-scale));
  }
  thead {
    position: sticky;
    top: 0;
    background: var(--surface-2);
    z-index: 1;
  }
  th {
    text-align: left;
    padding: 8px 8px;
    color: var(--text-muted);
    font-weight: 600;
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
  }
  th.star-col { width: 32px; text-align: center; }
  th.code-col { width: 80px; }
  th.price-col, th.pct-col { text-align: right; }
  td {
    padding: 6px 8px;
    border-bottom: 1px solid var(--border-soft);
    color: var(--text);
  }
  .etf-row {
    cursor: pointer;
    transition: background 0.12s;
  }
  .etf-row:hover {
    background: var(--row-hover);
  }
  .etf-row.is-favorite {
    background: var(--accent-bg);
  }
  .etf-row.is-favorite:hover {
    background: var(--accent-bg-hover);
  }
  .star-cell {
    text-align: center;
    font-size: calc(1rem * var(--font-scale));
  }
  .star {
    color: var(--warning);
  }
  .etf-row:not(.is-favorite) .star {
    color: var(--text-dim);
  }
  .name-cell {
    font-weight: 500;
  }
  .code-cell {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: calc(0.8rem * var(--font-scale));
  }
  .price-cell {
    text-align: right;
    font-family: var(--font-mono);
    font-size: calc(0.82rem * var(--font-scale));
  }
  .pct-cell {
    text-align: right;
    font-family: var(--font-mono);
    font-size: calc(0.82rem * var(--font-scale));
    white-space: nowrap;
  }
  .panel-footer {
    display: flex;
    justify-content: space-between;
    font-size: calc(0.78rem * var(--font-scale));
    color: var(--text-muted);
  }
</style>
