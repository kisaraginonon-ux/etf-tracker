// PositionSummary — 전체 가상 포지션 요약 뷰
// 모든 포지션 테이블 + 총 평가손익/투자금액/수익률 요약
// 행 클릭 → selectTicker (좌측 상세 패널로 전환)

<script lang="ts">
  import { positions, etfList, manualQuotes, selectTicker } from '$lib/stores';
  import type { VirtualPosition, EtfListItem, NormalizedQuote } from '$lib/types';

  // 현재가 조회: manualQuotes 우선, etfList에서 찾기 차선
  function getCurrentPrice(ticker: string): number | null {
    const mq = $manualQuotes;
    const quote: NormalizedQuote | undefined = mq.get(ticker);
    if (quote !== undefined) {
      return quote.current_price;
    }
    const list = $etfList;
    const item: EtfListItem | undefined = list.find((i: EtfListItem) => i.ticker === ticker);
    if (item !== undefined) {
      return item.current_price;
    }
    return null;
  }

  function getEtfName(ticker: string): string {
    const list = $etfList;
    const item: EtfListItem | undefined = list.find((i: EtfListItem) => i.ticker === ticker);
    if (item !== undefined) {
      return item.name;
    }
    // etfList에 없으면 ticker를 이름으로 사용
    return ticker;
  }

  // 포지션별 평가손익 계산
  interface PositionRow {
    ticker: string;
    name: string;
    buy_date: string;
    avg_price: number;
    quantity: number;
    current_price: number | null;
    eval_profit: number | null;
    eval_pct: number | null;
    invest_amount: number;
  }

  let positionRows = $derived(
    (() => {
      const raw = $positions;
      const rows: PositionRow[] = [];
      for (const p of raw) {
        const avg = p.avg_buy_price !== null ? p.avg_buy_price : 0;
        const qty = p.quantity !== null ? p.quantity : 0;
        const cur = getCurrentPrice(p.ticker);
        const invest = avg * qty;
        let evalProfit: number | null = null;
        let evalPct: number | null = null;
        if (cur !== null && avg > 0 && qty > 0) {
          evalProfit = (cur - avg) * qty;
          evalPct = ((cur - avg) / avg) * 100;
        }
        rows.push({
          ticker: p.ticker,
          name: getEtfName(p.ticker),
          buy_date: p.buy_date !== null ? p.buy_date : '-',
          avg_price: avg,
          quantity: qty,
          current_price: cur,
          eval_profit: evalProfit,
          eval_pct: evalPct,
          invest_amount: invest,
        });
      }
      return rows;
    })()
  );

  // 요약 계산
  let totalInvest = $derived(
    positionRows.reduce((sum: number, r: PositionRow) => sum + r.invest_amount, 0)
  );
  let totalEvalProfit = $derived(
    positionRows.reduce(
      (sum: number, r: PositionRow) => sum + (r.eval_profit !== null ? r.eval_profit : 0),
      0
    )
  );
  let totalEvalValue = $derived(totalInvest + totalEvalProfit);
  let totalPct = $derived(
    totalInvest > 0 ? (totalEvalProfit / totalInvest) * 100 : 0
  );

  function formatPrice(n: number): string {
    if (n === 0) return '-';
    return n.toLocaleString('ko-KR');
  }

  function formatPriceNullable(n: number | null): string {
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

  function onRowClick(ticker: string) {
    selectTicker(ticker);
  }
</script>

<div class="position-summary">
  {#if positionRows.length === 0}
    <div class="empty-state">
      <p class="empty-icon">📊</p>
      <p class="empty-text">가상 포지션이 없습니다</p>
      <p class="empty-hint">좌측 상세 패널에서 종목을 선택하고<br />가상 포지션을 입력하면 여기에 표시됩니다.</p>
    </div>
  {:else}
    <div class="table-scroll">
      <table class="position-table">
        <thead>
          <tr>
            <th class="name-col">종목명</th>
            <th class="date-col">매수일</th>
            <th class="num-col">평균단가</th>
            <th class="num-col">수량</th>
            <th class="num-col">현재가</th>
            <th class="num-col">평가손익</th>
            <th class="num-col">수익률</th>
          </tr>
        </thead>
        <tbody>
          {#each positionRows as row (row.ticker)}
            <tr
              class="pos-row"
              onclick={() => onRowClick(row.ticker)}
              title="클릭하여 상세 정보 보기"
            >
              <td class="name-cell">{row.name}</td>
              <td class="date-cell">{row.buy_date}</td>
              <td class="num-cell">{formatPrice(row.avg_price)}</td>
              <td class="num-cell">{row.quantity.toLocaleString('ko-KR')}</td>
              <td class="num-cell">{formatPriceNullable(row.current_price)}</td>
              <td class="num-cell" style="color: {colorForChange(row.eval_profit)}; font-weight: 600;">
                {formatPriceNullable(row.eval_profit)}
              </td>
              <td class="num-cell" style="color: {colorForChange(row.eval_pct)}; font-weight: 600;">
                {formatPct(row.eval_pct)}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="summary-bar">
      <div class="summary-item">
        <span class="summary-label">총 투자금액</span>
        <span class="summary-value">{formatPrice(totalInvest)}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">총 평가금액</span>
        <span class="summary-value">{formatPrice(totalEvalValue)}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">총 평가손익</span>
        <span class="summary-value" style="color: {colorForChange(totalEvalProfit)}; font-weight: 700;">
          {totalEvalProfit >= 0 ? '+' : ''}{formatPrice(totalEvalProfit)}
        </span>
      </div>
      <div class="summary-item">
        <span class="summary-label">전체 수익률</span>
        <span class="summary-value" style="color: {colorForChange(totalPct)}; font-weight: 700;">
          {totalPct >= 0 ? '+' : ''}{totalPct.toFixed(2)}%
        </span>
      </div>
    </div>
  {/if}
</div>

<style>
  .position-summary {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow: hidden;
  }
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    color: var(--text-dim);
    gap: 8px;
    padding: 40px 20px;
  }
  .empty-icon {
    font-size: calc(2.4rem * var(--font-scale));
    margin: 0;
  }
  .empty-text {
    font-size: calc(1.1rem * var(--font-scale));
    color: var(--text-secondary);
    margin: 0;
  }
  .empty-hint {
    font-size: calc(0.85rem * var(--font-scale));
    color: var(--text-muted);
    margin: 0;
    line-height: 1.6;
  }
  .table-scroll {
    flex: 1;
    overflow: auto;
    border: 1px solid var(--border-soft);
    border-radius: 6px;
  }
  .position-table {
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
  th.date-col { min-width: 90px; }
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
  .name-cell {
    font-weight: 500;
  }
  .date-cell {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: calc(0.82rem * var(--font-scale));
  }
  .pos-row {
    cursor: pointer;
  }
  .pos-row:hover {
    background: var(--row-hover);
  }
  .summary-bar {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    padding: 10px 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
  }
  .summary-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .summary-label {
    font-size: calc(0.75rem * var(--font-scale));
    color: var(--text-muted);
  }
  .summary-value {
    font-size: calc(1rem * var(--font-scale));
    color: var(--text);
    font-family: var(--font-mono);
    font-weight: 600;
  }
</style>