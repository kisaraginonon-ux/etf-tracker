// EtfGrid — 전체 ETF 목록 그리드
// etfList 기반, 즐겨찾기 상단 고정, 정렬, 필터, 행 클릭→선택, ★ 즐겨찾기 토글
// 기능 1: 장중 자동 새로고침 (regular 상태 시 2분 간격)
// 기능 5: 컬럼 선택 기능 (⚙️ 버튼 → 드롭다운 토글, localStorage 저장)
// 기능 6: 포지션 입력 종목 하이라이트

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { etfList, etfListLoading, etfListError, loadEtfList, favorites, addFavoriteAction, removeFavoriteAction, selectTicker, selectedTicker, positions, marketState } from '$lib/stores';
  import type { EtfListItem, Favorite, VirtualPosition } from '$lib/types';

  let filterText = $state('');
  let sortKey = $state<'name' | 'current_price' | 'change_pct' | 'volume'>('name');
  let sortAsc = $state(true);
  let favoriteFirst = $state(true);

  let favoriteTickers = $derived(new Set($favorites.map((f: Favorite) => f.ticker)));
  let positionTickers = $derived(new Set($positions.map((p: VirtualPosition) => p.ticker)));

  // === 컬럼 선택 기능 (기능 5) ===
  type ColumnKey = 'trading_value' | 'prev_close' | 'change_amount';
  const STORAGE_KEY = 'etf-grid-columns';

  // localStorage에서 복원
  function loadColumnPrefs(): Record<ColumnKey, boolean> {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw !== null) {
        const parsed = JSON.parse(raw) as Record<string, boolean>;
        return {
          trading_value: parsed.trading_value === true,
          prev_close: parsed.prev_close === true,
          change_amount: parsed.change_amount === true,
        };
      }
    } catch (e) {
      console.error('Failed to load column prefs:', e);
    }
    return { trading_value: false, prev_close: false, change_amount: false };
  }

  let columnPrefs = $state<Record<ColumnKey, boolean>>(loadColumnPrefs());
  let showColumnMenu = $state(false);

  function toggleColumn(key: ColumnKey) {
    columnPrefs[key] = !columnPrefs[key];
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(columnPrefs));
    } catch (e) {
      console.error('Failed to save column prefs:', e);
    }
  }

  function toggleColumnMenu() {
    showColumnMenu = !showColumnMenu;
  }

  // 외부 클릭 시 드롭다운 닫기
  function handleDocClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.closest('.column-menu-wrap') === null && showColumnMenu) {
      showColumnMenu = false;
    }
  }

  // === 장중 자동 새로고침 (기능 1) ===
  let autoRefreshTimer: ReturnType<typeof setInterval> | null = null;
  const AUTO_REFRESH_INTERVAL = 2 * 60 * 1000; // 2분

  function startAutoRefresh() {
    if (autoRefreshTimer !== null) return;
    autoRefreshTimer = setInterval(() => {
      if ($marketState === 'regular') {
        loadEtfList();
      }
    }, AUTO_REFRESH_INTERVAL);
  }

  function stopAutoRefresh() {
    if (autoRefreshTimer !== null) {
      clearInterval(autoRefreshTimer);
      autoRefreshTimer = null;
    }
  }

  onMount(() => {
    // 최초 1회 로드는 +page.svelte에서 수행
    document.addEventListener('click', handleDocClick);
    startAutoRefresh();
  });

  onDestroy(() => {
    stopAutoRefresh();
    document.removeEventListener('click', handleDocClick);
  });

  let filteredList = $derived(
    (() => {
      let list = $etfList;

      // 필터
      if (filterText.trim() !== '') {
        const q = filterText.trim().toLowerCase();
        list = list.filter((item: EtfListItem) =>
          item.name.toLowerCase().indexOf(q) !== -1 ||
          item.ticker.toLowerCase().indexOf(q) !== -1
        );
      }

      // 단일 comparator: 즐겨찾기 우선 → sortKey
      let sorted = [...list];
      sorted.sort((a: EtfListItem, b: EtfListItem) => {
        // 1순위: 즐겨찾기 상단 고정
        if (favoriteFirst) {
          const aFav = favoriteTickers.has(a.ticker) ? 0 : 1;
          const bFav = favoriteTickers.has(b.ticker) ? 0 : 1;
          if (aFav !== bFav) return aFav - bFav;
        }
        // 2순위: 선택된 정렬키
        const av: string | number = a[sortKey];
        const bv: string | number = b[sortKey];
        if (typeof av === 'string' && typeof bv === 'string') {
          return sortAsc ? av.localeCompare(bv) : bv.localeCompare(av);
        }
        return sortAsc ? (av as number) - (bv as number) : (bv as number) - (av as number);
      });

      return sorted;
    })()
  );

  function toggleSort(key: typeof sortKey) {
    if (sortKey === key) {
      sortAsc = !sortAsc;
    } else {
      sortKey = key;
      sortAsc = true;
    }
  }

  async function onToggleFavorite(item: EtfListItem, e: Event) {
    e.stopPropagation();
    if (favoriteTickers.has(item.ticker)) {
      await removeFavoriteAction(item.ticker);
    } else {
      await addFavoriteAction({
        ticker: item.ticker,
        name: item.name,
        market_section: '',
        is_active: true,
        added_at: '',
      } as Favorite);
    }
  }

  function onSelectRow(item: EtfListItem) {
    selectTicker(item.ticker);
  }

  async function onRefresh() {
    await loadEtfList();
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

  const sortIcon = (key: typeof sortKey) => {
    if (sortKey !== key) return '⇅';
    return sortAsc ? '↑' : '↓';
  };
</script>

<div class="grid-container">
  <div class="grid-toolbar">
    <input
      class="filter-input"
      type="text"
      placeholder="🔍 종목명 또는 코드 검색"
      bind:value={filterText}
    />
    <div class="sort-controls">
      <label class="sort-label">
        <input type="checkbox" bind:checked={favoriteFirst} />
        즐겨찾기 상단 고정
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
    <div class="action-controls">
      <!-- 컬럼 선택 버튼 (기능 5) -->
      <div class="column-menu-wrap">
        <button class="refresh-btn" onclick={toggleColumnMenu} title="표시할 컬럼 선택">
          ⚙️ 컬럼
        </button>
        {#if showColumnMenu}
          <div class="column-dropdown">
            <p class="dropdown-title">표시할 컬럼</p>
            <label class="dropdown-item">
              <input type="checkbox" checked={columnPrefs.trading_value} onchange={() => toggleColumn('trading_value')} />
              거래대금
            </label>
            <label class="dropdown-item">
              <input type="checkbox" checked={columnPrefs.prev_close} onchange={() => toggleColumn('prev_close')} />
              전일종가
            </label>
            <label class="dropdown-item">
              <input type="checkbox" checked={columnPrefs.change_amount} onchange={() => toggleColumn('change_amount')} />
              등락액
            </label>
          </div>
        {/if}
      </div>
      <button class="refresh-btn" onclick={onRefresh} disabled={$etfListLoading} title="ETF 목록 새로고침">
        {#if $etfListLoading}🔄 불러오는 중...{:else}🔄 새로고침{/if}
      </button>
    </div>
  </div>

  {#if $etfListLoading && $etfList.length === 0}
    <div class="state-message">
      <p>⏳ ETF 목록을 불러오는 중...</p>
    </div>
  {:else if $etfListError !== null}
    <div class="state-message error">
      <p>❌ 목록 조회 실패: {$etfListError}</p>
      <button class="retry-btn" onclick={onRefresh}>다시 시도</button>
    </div>
  {:else if filteredList.length === 0}
    <div class="state-message">
      <p>📋 {#if filterText.trim() !== ''}검색 결과가 없습니다.{:else}ETF 목록이 비어 있습니다.{/if}</p>
      {#if filterText.trim() !== ''}
        <button class="retry-btn" onclick={() => filterText = ''}>필터 초기화</button>
      {/if}
    </div>
  {:else}
    <div class="list-scroll">
      <table class="grid">
        <thead>
          <tr>
            <th class="fav-col">★</th>
            <th class="name-col">종목명</th>
            <th class="ticker-col">코드</th>
            <th class="num-col">현재가</th>
            <th class="num-col">등락률</th>
            <th class="num-col">거래량</th>
            {#if columnPrefs.trading_value}
              <th class="num-col">거래대금</th>
            {/if}
            {#if columnPrefs.prev_close}
              <th class="num-col">전일종가</th>
            {/if}
            {#if columnPrefs.change_amount}
              <th class="num-col">등락액</th>
            {/if}
          </tr>
        </thead>
        <tbody>
          {#each filteredList as item (item.ticker)}
            <tr
              class="grid-row"
              class:is-favorite={favoriteTickers.has(item.ticker)}
              class:is-selected={$selectedTicker === item.ticker}
              class:has-position={positionTickers.has(item.ticker)}
              onclick={() => onSelectRow(item)}
            >
              <td class="fav-cell">
                <button
                  class="fav-toggle"
                  class:active={favoriteTickers.has(item.ticker)}
                  onclick={(e) => onToggleFavorite(item, e)}
                  title={favoriteTickers.has(item.ticker) ? '즐겨찾기 해제' : '즐겨찾기 추가'}
                >
                  {favoriteTickers.has(item.ticker) ? '★' : '☆'}
                </button>
              </td>
              <td class="name-cell">{item.name}</td>
              <td class="ticker-cell">{item.ticker}</td>
              <td class="num-cell">{formatPrice(item.current_price)}</td>
              <td class="num-cell" style="color: {colorForChange(item.change_pct)}">
                {formatPct(item.change_pct)}
              </td>
              <td class="num-cell">{formatPrice(item.volume)}</td>
              {#if columnPrefs.trading_value}
                <td class="num-cell">{formatPrice(item.trading_value)}</td>
              {/if}
              {#if columnPrefs.prev_close}
                <td class="num-cell">{formatPrice(item.prev_close)}</td>
              {/if}
              {#if columnPrefs.change_amount}
                <td class="num-cell" style="color: {colorForChange(item.change_amount)}">
                  {item.change_amount >= 0 ? '+' : ''}{formatPrice(item.change_amount)}
                </td>
              {/if}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="grid-footer">
      <span class="count-info">
        총 {filteredList.length}개
        {#if favoriteTickers.size > 0} · 즐겨찾기 {filteredList.filter((i: EtfListItem) => favoriteTickers.has(i.ticker)).length}개{/if}
        {#if positionTickers.size > 0} · 포지션 {filteredList.filter((i: EtfListItem) => positionTickers.has(i.ticker)).length}개{/if}
      </span>
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
    gap: 12px;
    flex-wrap: wrap;
  }
  .filter-input {
    flex: 1;
    min-width: 160px;
    background: var(--surface-input);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 10px;
    font-size: calc(0.88rem * var(--font-scale));
  }
  .filter-input:focus {
    border-color: var(--accent);
    outline: none;
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
    font-size: calc(0.82rem * var(--font-scale));
    color: var(--text-secondary);
    cursor: pointer;
  }
  .sort-divider {
    color: var(--text-dim);
  }
  .sort-title {
    font-size: calc(0.82rem * var(--font-scale));
    color: var(--text-muted);
  }
  .sort-btn {
    background: var(--surface-3);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 3px 10px;
    cursor: pointer;
    font-size: calc(0.82rem * var(--font-scale));
  }
  .sort-btn:hover {
    background: var(--surface-hover);
  }
  .sort-btn.active {
    background: var(--accent-bg);
    color: var(--accent);
    border-color: var(--accent-border);
  }
  .action-controls {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .refresh-btn {
    background: var(--surface-3);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 5px 14px;
    cursor: pointer;
    font-size: calc(0.82rem * var(--font-scale));
  }
  .refresh-btn:hover:not(:disabled) {
    background: var(--surface-hover);
  }
  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  /* === 컬럼 선택 드롭다운 (기능 5) === */
  .column-menu-wrap {
    position: relative;
  }
  .column-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: var(--shadow);
    padding: 10px 12px;
    z-index: 100;
    min-width: 160px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .dropdown-title {
    margin: 0 0 4px 0;
    font-size: calc(0.82rem * var(--font-scale));
    color: var(--text-muted);
    font-weight: 600;
  }
  .dropdown-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: calc(0.85rem * var(--font-scale));
    color: var(--text);
    cursor: pointer;
    user-select: none;
  }
  .dropdown-item input[type="checkbox"] {
    cursor: pointer;
  }
  .state-message {
    text-align: center;
    padding: 40px 20px;
    color: var(--text-dim);
    font-size: calc(0.9rem * var(--font-scale));
  }
  .state-message.error {
    color: var(--danger);
  }
  .retry-btn {
    margin-top: 10px;
    background: var(--accent-bg);
    color: var(--accent);
    border: 1px solid var(--accent-border);
    border-radius: 6px;
    padding: 5px 14px;
    cursor: pointer;
    font-size: calc(0.85rem * var(--font-scale));
  }
  .retry-btn:hover {
    background: var(--accent-bg-hover);
  }
  .list-scroll {
    flex: 1;
    overflow: auto;
    border: 1px solid var(--border-soft);
    border-radius: 6px;
  }
  .grid {
    width: 100%;
    border-collapse: collapse;
    font-size: calc(0.88rem * var(--font-scale));
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
  th.fav-col { width: 40px; text-align: center; }
  th.ticker-col { min-width: 70px; }
  td {
    padding: 7px 10px;
    border-bottom: 1px solid var(--border-soft);
    color: var(--text);
  }
  td.num-cell {
    text-align: right;
    font-family: var(--font-mono);
    font-size: calc(0.85rem * var(--font-scale));
  }
  .fav-cell {
    text-align: center;
  }
  .fav-toggle {
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: calc(1rem * var(--font-scale));
    color: var(--text-dim);
    padding: 2px 4px;
    border-radius: 4px;
  }
  .fav-toggle:hover {
    color: var(--warning);
    background: var(--row-hover);
  }
  .fav-toggle.active {
    color: var(--warning);
  }
  .name-cell {
    font-weight: 500;
  }
  .ticker-cell {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: calc(0.8rem * var(--font-scale));
  }
  .grid-row {
    cursor: pointer;
  }
  .grid-row:hover {
    background: var(--row-hover);
  }
  .grid-row.is-favorite {
    background: var(--accent-bg);
  }
  .grid-row.is-favorite:hover {
    background: var(--accent-bg-hover);
  }
  /* 포지션 하이라이트 (기능 6) — 좌측 보더로 구분 */
  .grid-row.has-position {
    border-left: 3px solid var(--accent);
  }
  .grid-row.is-selected {
    background: var(--surface-selected, var(--accent-bg));
    box-shadow: inset 3px 0 0 var(--accent);
  }
  .grid-row.is-selected:hover {
    background: var(--surface-selected-hover, var(--accent-bg-hover));
  }
  .grid-footer {
    padding: 4px 8px;
    font-size: calc(0.8rem * var(--font-scale));
    color: var(--text-muted);
  }
  .count-info {
    display: inline;
  }
</style>