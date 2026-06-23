// QuoteGrid — ETF 데이터 그리드 (REQ-F-01~04)
// 열 정렬, 상태 배지, 즐겨찾기 관리

<script lang="ts">
  import type { Favorite, DataStatus } from '$lib/types';
  import { removeFavoriteAction } from '$lib/stores';
  import StatusBadge from './StatusBadge.svelte';

  let { favorites }: { favorites: Favorite[] } = $props();

  type SortKey = 'name' | 'ticker' | 'current_price' | 'change_pct' | 'volume' | 'status';
  let sortKey = $state<SortKey>('name');
  let sortAsc = $state(true);

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      sortAsc = !sortAsc;
    } else {
      sortKey = key;
      sortAsc = true;
    }
  }

  // 임시 더미 데이터 (Phase 5에서는 API 연동 전, UI 구조만)
  let rows = $derived(
    favorites.map(f => ({
      ticker: f.ticker,
      name: f.name,
      current_price: 0,
      change: 0,
      change_pct: 0,
      volume: 0,
      index_diff: 0,
      status: 'stale' as DataStatus,
    })).sort((a, b) => {
      const av = a[sortKey];
      const bv = b[sortKey];
      if (typeof av === 'string' && typeof bv === 'string') {
        return sortAsc ? av.localeCompare(bv) : bv.localeCompare(av);
      }
      return sortAsc
        ? (av as number) - (bv as number)
        : (bv as number) - (av as number);
    })
  );

  function onRemove(ticker: string) {
    removeFavoriteAction(ticker);
  }

  function formatNum(n: number): string {
    if (n === 0) return '-';
    return n.toLocaleString('ko-KR');
  }

  function formatPct(n: number): string {
    if (n === 0) return '-';
    return (n >= 0 ? '+' : '') + n.toFixed(2) + '%';
  }

  function colorForChange(n: number): string {
    if (n > 0) return '#ef5350'; // 상승 빨강 (한국식)
    if (n < 0) return '#26a69a'; // 하락 파랑
    return '#888';
  }

  const sortIcon = (key: SortKey) => {
    if (sortKey !== key) return '⇅';
    return sortAsc ? '↑' : '↓';
  };
</script>

<div class="grid-container">
  {#if rows.length === 0}
    <div class="empty-state">
      <p>📋 등록된 종목이 없습니다.</p>
      <p class="hint">위 검색창에서 ETF 종목을 검색하여 추가해주세요.</p>
    </div>
  {:else}
    <table class="grid">
      <thead>
        <tr>
          <th class="sortable" onclick={() => toggleSort('name')}>종목명 {sortIcon('name')}</th>
          <th class="sortable ticker-col" onclick={() => toggleSort('ticker')}>코드 {sortIcon('ticker')}</th>
          <th class="sortable num-col" onclick={() => toggleSort('current_price')}>현재가 {sortIcon('current_price')}</th>
          <th class="sortable num-col" onclick={() => toggleSort('change_pct')}>등락률 {sortIcon('change_pct')}</th>
          <th class="sortable num-col" onclick={() => toggleSort('volume')}>거래량 {sortIcon('volume')}</th>
          <th class="num-col">지수대비</th>
          <th class="sortable" onclick={() => toggleSort('status')}>상태 {sortIcon('status')}</th>
          <th class="action-col">-</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.ticker)}
          <tr>
            <td class="name-cell">{row.name}</td>
            <td class="ticker-cell">{row.ticker}</td>
            <td class="num-cell">{formatNum(row.current_price)}</td>
            <td class="num-cell" style="color: {colorForChange(row.change_pct)}">
              {formatPct(row.change_pct)}
            </td>
            <td class="num-cell">{formatNum(row.volume)}</td>
            <td class="num-cell" style="color: {colorForChange(row.index_diff)}">
              {formatPct(row.index_diff)}
            </td>
            <td class="status-cell"><StatusBadge status={row.status} /></td>
            <td class="action-cell">
              <button class="remove-btn" onclick={() => onRemove(row.ticker)} title="삭제">✕</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .grid-container { flex: 1; overflow: auto; }
  .empty-state { text-align: center; padding: 60px 20px; color: #666; }
  .empty-state .hint { font-size: 0.85rem; margin-top: 8px; }
  .grid { width: 100%; border-collapse: collapse; font-size: 0.88rem; }
  thead { position: sticky; top: 0; background: #1e1e2e; z-index: 1; }
  th {
    text-align: left; padding: 10px 12px; border-bottom: 2px solid #333;
    color: #999; font-weight: 600; white-space: nowrap; user-select: none;
  }
  th.sortable { cursor: pointer; }
  th.sortable:hover { color: #6366f1; }
  th.num-col { text-align: right; }
  th.ticker-col { min-width: 80px; }
  th.action-col { width: 40px; }
  td { padding: 8px 12px; border-bottom: 1px solid #252535; color: #e0e0e0; }
  td.num-cell { text-align: right; font-family: 'SF Mono', monospace; font-size: 0.85rem; }
  .name-cell { font-weight: 500; }
  .ticker-cell { color: #888; font-family: monospace; font-size: 0.8rem; }
  .status-cell { text-align: center; }
  .action-cell { text-align: center; }
  .remove-btn {
    background: transparent; border: none; color: #666; cursor: pointer;
    font-size: 0.9rem; padding: 2px 6px; border-radius: 4px;
  }
  .remove-btn:hover { color: #f44336; background: rgba(244,67,54,0.1); }
  tbody tr:hover { background: rgba(99,102,241,0.05); }
</style>