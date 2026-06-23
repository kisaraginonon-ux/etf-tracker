// PositionPanel — 가상 포지션 리스트 형식 동시 편집 (REQ-F-07, REQ-F-08)
// 모든 즐겨찾기 종목을 테이블로 한 번에 표시, 일괄 저장

<script lang="ts">
  import {
    positions,
    loadPositions,
    setPositionAction,
    removePositionAction,
    removeFavoriteAction,
    favorites,
    manualQuotes,
  } from '$lib/stores';
  import type { Favorite, VirtualPosition } from '$lib/types';

  // 행 편집 상태: ticker → { buy_date, avg_price, quantity, dirty }
  type RowState = {
    buy_date: string;
    avg_price: string;
    quantity: string;
    dirty: boolean;
  };

  let rowStates = $state<Record<string, RowState>>({});
  let saving = $state(false);
  let saveStatus = $state<'idle' | 'done' | 'error'>('idle');

  // 즐겨찾기 + 포지션 병합 행
  let rows = $derived(
    $favorites.map((fav: Favorite) => {
      const pos = $positions.find((p: VirtualPosition) => p.ticker === fav.ticker);
      const quote = $manualQuotes.get(fav.ticker);
      const avg = pos && pos.avg_buy_price ? pos.avg_buy_price : null;
      const qty = pos && pos.quantity ? pos.quantity : null;
      const current = quote ? quote.current_price : null;
      let evalProfit: number | null = null;
      let evalPct: number | null = null;
      if (avg !== null && qty !== null && current !== null) {
        evalProfit = (current - avg) * qty;
        if (avg > 0) {
          evalPct = ((current - avg) / avg) * 100;
        }
      }
      return {
        ticker: fav.ticker,
        name: fav.name,
        buy_date: pos && pos.buy_date ? pos.buy_date : null,
        avg_buy_price: avg,
        quantity: qty,
        eval_profit: evalProfit,
        eval_profit_pct: evalPct,
        current_price: current,
      };
    })
  );

  // rows 변경 시 rowStates 동기화 (최초 로드 또는 favorites/positions 변경)
  let lastSyncedKey = $state('');
  let syncKey = $derived(
    $favorites.map((f: Favorite) => f.ticker).join(',') +
    '|' +
    $positions.map((p: VirtualPosition) => p.ticker + ':' + (p.buy_date || '') + ':' + (p.avg_buy_price || 0) + ':' + (p.quantity || 0)).join(',')
  );

  $effect(() => {
    // syncKey가 변경되면 rowStates를 백엔드 상태 기준으로 재구성
    const _ = syncKey; // 의존성 트리거
    const newState: Record<string, RowState> = {};
    for (const fav of $favorites) {
      const pos = $positions.find((p: VirtualPosition) => p.ticker === fav.ticker);
      const existing = rowStates[fav.ticker];
      // 기존 dirty 상태 유지 (저장 전 사용자 입력 보존)
      if (existing && existing.dirty) {
        newState[fav.ticker] = existing;
      } else {
        newState[fav.ticker] = {
          buy_date: pos && pos.buy_date ? pos.buy_date : '',
          avg_price: pos && pos.avg_buy_price ? pos.avg_buy_price.toString() : '',
          quantity: pos && pos.quantity ? pos.quantity.toString() : '',
          dirty: false,
        };
      }
    }
    rowStates = newState;
    lastSyncedKey = syncKey;
  });

  function markDirty(ticker: string) {
    if (rowStates[ticker]) {
      rowStates[ticker] = { ...rowStates[ticker], dirty: true };
    }
  }

  function onBuyDateChange(ticker: string, value: string) {
    if (!rowStates[ticker]) return;
    rowStates[ticker] = { ...rowStates[ticker], buy_date: value, dirty: true };
  }

  function onAvgPriceChange(ticker: string, value: string) {
    if (!rowStates[ticker]) return;
    rowStates[ticker] = { ...rowStates[ticker], avg_price: value, dirty: true };
  }

  function onQtyChange(ticker: string, value: string) {
    if (!rowStates[ticker]) return;
    rowStates[ticker] = { ...rowStates[ticker], quantity: value, dirty: true };
  }

  let dirtyCount = $derived(
    Object.values(rowStates).filter((r: RowState) => r.dirty).length
  );

  async function saveAll() {
    saving = true;
    saveStatus = 'idle';
    try {
      const dirtyEntries = Object.entries(rowStates).filter(([_t, r]) => r.dirty);
      for (const [ticker, r] of dirtyEntries) {
        const buyDate = r.buy_date || null;
        const avg = r.avg_price ? parseFloat(r.avg_price) : null;
        const qty = r.quantity ? parseFloat(r.quantity) : null;
        await setPositionAction(ticker, buyDate, avg, qty);
        if (rowStates[ticker]) {
          rowStates[ticker] = { ...rowStates[ticker], dirty: false };
        }
      }
      saveStatus = 'done';
      setTimeout(() => { saveStatus = 'idle'; }, 2500);
    } catch (e) {
      console.error('일괄 저장 실패:', e);
      saveStatus = 'error';
      setTimeout(() => { saveStatus = 'idle'; }, 2500);
    } finally {
      saving = false;
    }
  }

  async function onDeleteRow(ticker: string) {
    await removePositionAction(ticker);
    // 상태에서도 제거
    if (rowStates[ticker]) {
      const next = { ...rowStates };
      delete next[ticker];
      rowStates = next;
    }
  }

  async function onRemoveFavorite(ticker: string) {
    await removeFavoriteAction(ticker);
  }

  function formatNum(n: number | null): string {
    if (n === null) return '-';
    return n.toLocaleString('ko-KR');
  }

  function formatPct(n: number | null): string {
    if (n === null) return '-';
    return (n >= 0 ? '+' : '') + n.toFixed(2) + '%';
  }

  function colorForChange(n: number | null): string {
    if (n === null) return 'var(--text-dim)';
    if (n > 0) return 'var(--color-up)';
    if (n < 0) return 'var(--color-down)';
    return 'var(--color-flat)';
  }
</script>

<div class="position-panel">
  <div class="header">
    <h3>📊 가상 포지션</h3>
    <span class="notice">참고용 가상 포지션 (실제 계좌 미연동)</span>
  </div>

  {#if rows.length === 0}
    <p class="empty">즐겨찾기에 종목을 추가하면 포지션을 입력할 수 있습니다.</p>
  {:else}
    <div class="table-wrap">
      <table class="pos-table">
        <thead>
          <tr>
            <th class="name-col">종목명</th>
            <th class="date-col">매수일</th>
            <th class="num-col">평균단가</th>
            <th class="num-col">수량</th>
            <th class="num-col">평가손익</th>
            <th class="num-col">수익률</th>
            <th class="action-col">삭제</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.ticker)}
            {@const st = rowStates[row.ticker]}
            <tr>
              <td class="name-cell">
                <div class="name-wrap">
                  <span class="name-text">{row.name}</span>
                  <span class="ticker-text">{row.ticker}</span>
                </div>
              </td>
              <td class="date-cell">
                <input
                  type="date"
                  class="input-date"
                  value={st ? st.buy_date : ''}
                  oninput={(e) => onBuyDateChange(row.ticker, (e.target as HTMLInputElement).value)}
                />
              </td>
              <td class="num-cell">
                <input
                  type="number"
                  class="input-num"
                  placeholder="단가"
                  value={st ? st.avg_price : ''}
                  oninput={(e) => onAvgPriceChange(row.ticker, (e.target as HTMLInputElement).value)}
                />
              </td>
              <td class="num-cell">
                <input
                  type="number"
                  class="input-num"
                  placeholder="수량"
                  value={st ? st.quantity : ''}
                  oninput={(e) => onQtyChange(row.ticker, (e.target as HTMLInputElement).value)}
                />
              </td>
              <td class="num-cell" style="color: {colorForChange(row.eval_profit)}">
                {formatNum(row.eval_profit)}
              </td>
              <td class="num-cell" style="color: {colorForChange(row.eval_profit_pct)}">
                {formatPct(row.eval_profit_pct)}
              </td>
              <td class="action-cell">
                <button
                  class="btn-del"
                  onclick={() => onDeleteRow(row.ticker)}
                  title="포지션 삭제"
                >✕</button>
                <button
                  class="btn-remove-fav"
                  onclick={() => onRemoveFavorite(row.ticker)}
                  title="즐겨찾기에서 제거"
                >☆</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="save-bar">
      <span class="dirty-info">
        {#if dirtyCount > 0}
          변경된 행: {dirtyCount}개
        {:else}
          변경사항 없음
        {/if}
      </span>
      <button
        class="btn-save-all"
        onclick={saveAll}
        disabled={saving || dirtyCount === 0}
      >
        {#if saving}
          저장 중...
        {:else if saveStatus === 'done'}
          ✅ 저장 완료
        {:else if saveStatus === 'error'}
          ❌ 저장 실패
        {:else}
          💾 일괄 저장
        {/if}
      </button>
    </div>
  {/if}
</div>

<style>
  .position-panel {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .header {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .header h3 {
    margin: 0;
    font-size: calc(1rem * var(--font-scale));
    color: var(--text);
  }
  .notice {
    font-size: calc(0.75rem * var(--font-scale));
    color: var(--text-muted);
    font-style: italic;
  }
  .empty {
    color: var(--text-dim);
    font-size: calc(0.88rem * var(--font-scale));
    text-align: center;
    padding: 20px;
  }
  .table-wrap {
    overflow-x: auto;
    border: 1px solid var(--border-soft);
    border-radius: 6px;
  }
  .pos-table {
    width: 100%;
    border-collapse: collapse;
    font-size: calc(0.82rem * var(--font-scale));
  }
  .pos-table th {
    text-align: left;
    padding: 8px 6px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border);
    font-weight: 600;
    white-space: nowrap;
    background: var(--surface-2);
  }
  .pos-table th.num-col { text-align: right; }
  .pos-table th.action-col { text-align: center; width: 64px; }
  .pos-table td {
    padding: 4px 6px;
    border-bottom: 1px solid var(--border-soft);
    color: var(--text);
  }
  .name-cell { font-weight: 500; }
  .name-wrap {
    display: flex;
    flex-direction: column;
  }
  .ticker-text {
    font-size: calc(0.72rem * var(--font-scale));
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
  .num-cell {
    text-align: right;
    font-family: var(--font-mono);
    font-size: calc(0.82rem * var(--font-scale));
  }
  .date-cell, .num-cell {
    white-space: nowrap;
  }
  .input-date, .input-num {
    width: 100%;
    min-width: 80px;
    background: var(--surface-input);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 4px 6px;
    font-size: calc(0.8rem * var(--font-scale));
    font-family: var(--font-mono);
  }
  .input-date:focus, .input-num:focus {
    border-color: var(--accent);
    outline: none;
  }
  .input-num { text-align: right; }
  .action-cell {
    text-align: center;
    white-space: nowrap;
  }
  .btn-del, .btn-remove-fav {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 2px 4px;
    font-size: calc(0.9rem * var(--font-scale));
    border-radius: 4px;
    color: var(--text-dim);
  }
  .btn-del:hover {
    color: var(--danger);
    background: var(--remove-hover-bg);
  }
  .btn-remove-fav:hover {
    color: var(--warning);
    background: var(--row-hover);
  }
  .save-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 4px;
  }
  .dirty-info {
    font-size: calc(0.8rem * var(--font-scale));
    color: var(--text-muted);
  }
  .btn-save-all {
    background: var(--accent);
    color: var(--text);
    border: none;
    border-radius: 6px;
    padding: 6px 18px;
    cursor: pointer;
    font-size: calc(0.88rem * var(--font-scale));
    font-weight: 600;
  }
  .btn-save-all:hover:not(:disabled) {
    background: var(--accent-hover);
  }
  .btn-save-all:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>