// QuoteGrid — ETF 데이터 그리드 (REQ-F-01~04)
// 즐겨찾기 우선 정렬, 수동 시세 조회, 전체 조회

<script lang="ts">
  import type { Favorite, DataStatus, NormalizedQuote } from '$lib/types';
  import { removeFavoriteAction, manualQuotes, fetchQuoteNowAction } from '$lib/stores';
  import StatusBadge from './StatusBadge.svelte';

  let { favorites }: { favorites: Favorite[] } = $props();

  type SortKey = 'name' | 'current_price' | 'change_pct' | 'volume';
  let sortKey = $state<SortKey>('name');
  let sortAsc = $state(true);
  let favoriteFirst = $state(true);
  let fetchingAll = $state(false);

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      sortAsc = !sortAsc;
    } else {
      sortKey = key;
      sortAsc = true;
    }
  }

  // 즐겨찾기 ticker 집합 (이미 favorites 자체이지만 명확화)
  let favoriteTickers = $derived(new Set(favorites.map((f: Favorite) => f.ticker)));

  // 각 종목의 조회된 시세
  function quoteFor(ticker: string): NormalizedQuote | undefined {
    return $manualQuotes.get(ticker);
  }

  type GridRow = {
    ticker: string;
    name: string;
    current_price: number;
    change_pct: number;
    volume: number;
    status: DataStatus;
    has_quote: boolean;
  };

  let rows = $derived(
    (() => {
      const base: GridRow[] = favorites.map((f: Favorite) => {
        const q = $manualQuotes.get(f.ticker);
        if (q) {
          return {
            ticker: f.ticker,
            name: f.name,
            current_price: q.current_price,
            change_pct: q.change_pct,
            volume: q.volume,
            status: 'live' as DataStatus,
            has_quote: true,
          };
        }
        return {
          ticker: f.ticker,
          name: f.name,
          current_price: 0,
          change_pct: 0,
          volume: 0,
          status: 'stale' as DataStatus,
          has_quote: false,
        };
      });

      base.sort((a: GridRow, b: GridRow) => {
        // 즐겨찾기 우선 (모두 즐겨찾기이므로 사실상 동일하지만, has_quote 우선 적용)
        if (favoriteFirst) {
          // 조회된 데이터가 있는 것을 위로
          if (a.has_quote && !b.has_quote) return -1;
          if (!a.has_quote && b.has_quote) return 1;
        }
        // 그 다음 선택된 정렬키
        const av: string | number = a[sortKey];
        const bv: string | number = b[sortKey];
        if (typeof av === 'string' && typeof bv === 'string') {
          return sortAsc ? av.localeCompare(bv) : bv.localeCompare(av);
        }
        return sortAsc
          ? (av as number) - (bv as number)
          : (bv as number) - (av as number);
      });

      return base;
    })()
  );

  function onRemove(ticker: string) {
    removeFavoriteAction(ticker);
  }

  async function onFetchQuote(ticker: string) {
    await fetchQuoteNowAction(ticker);
  }

  async function onFetchAll() {
    fetchingAll = true;
    try {
      const tickers = favorites.map((f: Favorite) => f.ticker);
      // 순차 조회 (백엔드 부하 방지)
      for (const t of tickers) {
        await fetchQuoteNowAction(t);
      }
    } catch (e) {
      console.error('전체 조회 실패:', e);
    } finally {
      fetchingAll = false;
    }
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
    if (n > 0) return 'var(--color-up)';
    if (n < 0) return 'var(--color-down)';
    return 'var(--color-flat)';
  }

  const sortIcon = (key: SortKey) => {
    if (sortKey !== key) return '⇅';
    return sortAsc ? '↑' : '↓';
  };
</script>

<div class="grid-container">
  <div class="grid-toolbar">
    <div class="sort-controls">
      <label class="sort-label">
        <input type="checkbox" bind:checked={favoriteFirst} />
        즐겨찾기 우선
      </label>
      <span class="sort-divider">|</span>
      <span class="sort-title">정렬:</span>
      <button class="sort-btn" class:active={sortKey === 'name'} onclick={() => toggleSort('name')}>
        종목명 {sortIcon('name')}
      </button>
      <button class="sort-btn" class:active={sortKey === 'current_price'} onclick={() => toggleSort('current_price')}>
        현재가 {sortIcon('current_price')}
      </button>
      <button class="sort-btn" class:active={sortKey === 'change_pct'} onclick={() => toggleSort('change_pct')}>
        등락률 {sortIcon('change_pct')}
      </button>
      <button class="sort-btn" class:active={sortKey === 'volume'} onclick={() => toggleSort('volume')}>
        거래량 {sortIcon('volume')}
      </button>
    </div>
    <button class="fetch-all-btn" onclick={onFetchAll} disabled={fetchingAll || favorites.length === 0}>
      {#if fetchingAll}조회 중...{:else}⚡ 전체 조회{/if}
    </button>
  </div>

  {#if rows.length === 0}
    <div class="empty-state">
      <p>📋 등록된 종목이 없습니다.</p>
      <p class="hint">좌측 ETF 목록에서 종목을 클릭하여 즐겨찾기에 추가하세요.</p>
    </div>
  {:else}
    <div class="table-wrap">
      <table class="grid">
        <thead>
          <tr>
            <th class="name-col">종목명</th>
            <th class="ticker-col">코드</th>
            <th class="num-col">현재가</th>
            <th class="num-col">등락률</th>
            <th class="num-col">거래량</th>
            <th class="status-col">상태</th>
            <th class="action-col">조회</th>
            <th class="remove-col">삭제</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.ticker)}
            <tr class:has-quote={row.has_quote}>
              <td class="name-cell">{row.name}</td>
              <td class="ticker-cell">{row.ticker}</td>
              <td class="num-cell">{formatNum(row.current_price)}</td>
              <td class="num-cell" style="color: {colorForChange(row.change_pct)}">
                {formatPct(row.change_pct)}
              </td>
              <td class="num-cell">{formatNum(row.volume)}</td>
              <td class="status-cell"><StatusBadge status={row.status} /></td>
              <td class="action-cell">
                <button class="fetch-btn" onclick={() => onFetchQuote(row.ticker)} title="수동 시세 조회">
                  조회
                </button>
              </td>
              <td class="remove-cell">
                <button class="remove-btn" onclick={() => onRemove(row.ticker)} title="즐겨찾기 제거">✕</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .grid-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow: hidden;
  }
  .grid-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }
  .sort-controls {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .sort-label {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: calc(0.8rem * var(--font-scale));
    color: var(--text-secondary);
    cursor: pointer;
  }
  .sort-divider {
    color: var(--text-dim);
  }
  .sort-title {
    font-size: calc(0.8rem * var(--font-scale));
    color: var(--text-muted);
  }
  .sort-btn {
    background: var(--surface-3);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 3px 10px;
    cursor: pointer;
    font-size: calc(0.8rem * var(--font-scale));
  }
  .sort-btn:hover {
    background: var(--surface-hover);
  }
  .sort-btn.active {
    background: var(--accent-bg);
    color: var(--accent);
    border-color: var(--accent-border);
  }
  .fetch-all-btn {
    background: var(--accent);
    color: var(--text);
    border: none;
    border-radius: 6px;
    padding: 5px 14px;
    cursor: pointer;
    font-size: calc(0.82rem * var(--font-scale));
    font-weight: 600;
  }
  .fetch-all-btn:hover:not(:disabled) {
    background: var(--accent-hover);
  }
  .fetch-all-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .empty-state {
    text-align: center;
    padding: 40px 20px;
    color: var(--text-dim);
  }
  .empty-state .hint {
    font-size: calc(0.85rem * var(--font-scale));
    margin-top: 8px;
  }
  .table-wrap {
    flex: 1;
    overflow: auto;
    border: 1px solid var(--border-soft);
    border-radius: 6px;
  }
  .grid {
    width: 100%;
    border-collapse: collapse;
    font-size: calc(0.86rem * var(--font-scale));
  }
  thead {
    position: sticky;
    top: 0;
    background: var(--surface-2);
    z-index: 1;
  }
  th {
    text-align: left;
    padding: 8px 10px;
    border-bottom: 2px solid var(--border);
    color: var(--text-muted);
    font-weight: 600;
    white-space: nowrap;
    user-select: none;
  }
  th.num-col { text-align: right; }
  th.ticker-col { min-width: 70px; }
  th.status-col { text-align: center; min-width: 60px; }
  th.action-col { width: 56px; text-align: center; }
  th.remove-col { width: 40px; text-align: center; }
  td {
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-soft);
    color: var(--text);
  }
  td.num-cell {
    text-align: right;
    font-family: var(--font-mono);
    font-size: calc(0.83rem * var(--font-scale));
  }
  .name-cell { font-weight: 500; }
  .ticker-cell {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: calc(0.78rem * var(--font-scale));
  }
  .status-cell { text-align: center; }
  .action-cell { text-align: center; }
  .remove-cell { text-align: center; }
  .fetch-btn {
    background: var(--accent-bg);
    color: var(--accent);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 3px 10px;
    cursor: pointer;
    font-size: calc(0.78rem * var(--font-scale));
  }
  .fetch-btn:hover {
    background: var(--accent);
    color: var(--text);
  }
  .remove-btn {
    background: transparent;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    font-size: calc(0.9rem * var(--font-scale));
    padding: 2px 6px;
    border-radius: 4px;
  }
  .remove-btn:hover {
    color: var(--danger);
    background: var(--remove-hover-bg);
  }
  tr.has-quote {
    background: var(--accent-bg);
  }
  tr.has-quote:hover {
    background: var(--accent-bg-hover);
  }
  tbody tr:hover {
    background: var(--row-hover);
  }
</style>