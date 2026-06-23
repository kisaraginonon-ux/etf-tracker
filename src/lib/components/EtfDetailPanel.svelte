// EtfDetailPanel — 선택 종목 상세 패널
// 기간별 등락률, 현재가/거래량, 가상 포지션 입력

<script lang="ts">
  import {
    selectedTicker,
    periodReturns,
    periodReturnsLoading,
    periodReturnsError,
    positions,
    setPositionAction,
    selectTicker,
    etfList,
    manualQuotes,
  } from '$lib/stores';
  import type { VirtualPosition, EtfListItem, NormalizedQuote } from '$lib/types';

  // 선택된 종목의 포지션
  let currentPos = $derived(
    $selectedTicker !== null
      ? $positions.find((p: VirtualPosition) => p.ticker === $selectedTicker)
      : undefined
  );

  let buyDate = $state('');
  let avgPrice = $state('');
  let qty = $state('');
  let saving = $state(false);
  let saveStatus = $state<'idle' | 'done' | 'error'>('idle');

  // 포지션 또는 선택 종목 변경 시 입력 필드 동기화
  $effect(() => {
    const _ticker = $selectedTicker;
    const _positions = $positions;
    const p = _ticker !== null
      ? _positions.find((pp: VirtualPosition) => pp.ticker === _ticker)
      : undefined;
    buyDate = p && p.buy_date ? p.buy_date : '';
    avgPrice = p && p.avg_buy_price ? p.avg_buy_price.toString() : '';
    qty = p && p.quantity ? p.quantity.toString() : '';
  });

  // 평가손익 계산 (periodReturns의 current_price 사용)
  let evalProfit = $derived(
    avgPrice !== '' && qty !== '' && $periodReturns !== null
      ? ($periodReturns.current_price - parseFloat(avgPrice)) * parseFloat(qty)
      : null
  );
  let evalPct = $derived(
    avgPrice !== '' && $periodReturns !== null && parseFloat(avgPrice) > 0
      ? (($periodReturns.current_price - parseFloat(avgPrice)) / parseFloat(avgPrice)) * 100
      : null
  );

  async function onSavePosition() {
    if ($selectedTicker === null) return;
    saving = true;
    saveStatus = 'idle';
    try {
      const bd = buyDate !== '' ? buyDate : null;
      const ap = avgPrice !== '' ? parseFloat(avgPrice) : null;
      const q = qty !== '' ? parseFloat(qty) : null;
      await setPositionAction($selectedTicker, bd, ap, q);
      saveStatus = 'done';
      setTimeout(() => { saveStatus = 'idle'; }, 2500);
    } catch (e) {
      console.error('포지션 저장 실패:', e);
      saveStatus = 'error';
      setTimeout(() => { saveStatus = 'idle'; }, 2500);
    } finally {
      saving = false;
    }
  }

  function onClearSelection() {
    selectTicker(null);
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

  function formatNum(n: number | null): string {
    if (n === null) return '-';
    return n.toLocaleString('ko-KR');
  }

  function formatPctNullable(n: number | null): string {
    if (n === null) return '-';
    return (n >= 0 ? '+' : '') + n.toFixed(2) + '%';
  }

  function colorForChangeNullable(n: number | null): string {
    if (n === null) return 'var(--text-dim)';
    if (n > 0) return 'var(--color-up)';
    if (n < 0) return 'var(--color-down)';
    return 'var(--color-flat)';
  }

  // === 전체 포지션 요약 (빈 상태용) ===
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
    return ticker;
  }

  interface SummaryRow {
    ticker: string;
    name: string;
    buy_date: string;
    avg_price: number;
    quantity: number;
    eval_profit: number | null;
    eval_pct: number | null;
    invest_amount: number;
  }

  let summaryRows = $derived(
    (() => {
      const raw = $positions;
      const rows: SummaryRow[] = [];
      for (const p of raw) {
        const avg = p.avg_buy_price !== null ? p.avg_buy_price : 0;
        const qty = p.quantity !== null ? p.quantity : 0;
        const cur = getCurrentPrice(p.ticker);
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
          eval_profit: evalProfit,
          eval_pct: evalPct,
          invest_amount: avg * qty,
        });
      }
      return rows;
    })()
  );

  let totalInvest = $derived(
    summaryRows.reduce((sum: number, r: SummaryRow) => sum + r.invest_amount, 0)
  );
  let totalEvalProfit = $derived(
    summaryRows.reduce(
      (sum: number, r: SummaryRow) => sum + (r.eval_profit !== null ? r.eval_profit : 0),
      0
    )
  );
  let totalPct = $derived(
    totalInvest > 0 ? (totalEvalProfit / totalInvest) * 100 : 0
  );

  function onSummaryRowClick(ticker: string) {
    selectTicker(ticker);
  }

  function formatPriceNullable(n: number | null): string {
    if (n === null) return '-';
    return n.toLocaleString('ko-KR');
  }
</script>

<div class="detail-panel">
  {#if $selectedTicker === null}
    {#if summaryRows.length > 0}
      <!-- 포트폴리오 요약 뷰 (포지션이 있을 때) -->
      <section class="section portfolio-section">
        <h3 class="section-title">📊 포트폴리오 요약</h3>
        <div class="summary-table-wrap">
          <table class="summary-table">
            <thead>
              <tr>
                <th class="name-col">종목명</th>
                <th class="date-col">매수일</th>
                <th class="num-col">평균단가</th>
                <th class="num-col">수량</th>
                <th class="num-col">평가손익</th>
                <th class="num-col">수익률</th>
              </tr>
            </thead>
            <tbody>
              {#each summaryRows as row (row.ticker)}
                <tr
                  class="summary-row"
                  onclick={() => onSummaryRowClick(row.ticker)}
                  title="클릭하여 상세 정보 보기"
                >
                  <td class="name-cell">{row.name}</td>
                  <td class="date-cell">{row.buy_date}</td>
                  <td class="num-cell">{formatPrice(row.avg_price)}</td>
                  <td class="num-cell">{row.quantity.toLocaleString('ko-KR')}</td>
                  <td class="num-cell" style="color: {colorForChangeNullable(row.eval_profit)}; font-weight: 600;">
                    {formatPriceNullable(row.eval_profit)}
                  </td>
                  <td class="num-cell" style="color: {colorForChangeNullable(row.eval_pct)}; font-weight: 600;">
                    {formatPctNullable(row.eval_pct)}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
        <div class="portfolio-summary-bar">
          <div class="summary-item">
            <span class="summary-label">총 투자금액</span>
            <span class="summary-value">{formatPrice(totalInvest)}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">총 평가손익</span>
            <span class="summary-value" style="color: {colorForChangeNullable(totalEvalProfit)}; font-weight: 700;">
              {totalEvalProfit >= 0 ? '+' : ''}{formatPrice(totalEvalProfit)}
            </span>
          </div>
          <div class="summary-item">
            <span class="summary-label">전체 수익률</span>
            <span class="summary-value" style="color: {colorForChangeNullable(totalPct)}; font-weight: 700;">
              {totalPct >= 0 ? '+' : ''}{totalPct.toFixed(2)}%
            </span>
          </div>
        </div>
      </section>
    {:else}
      <div class="empty-state">
        <p class="empty-icon">👉</p>
        <p class="empty-text">종목을 선택하세요</p>
        <p class="empty-hint">우측 ETF 목록에서 종목 행을 클릭하면<br />상세 정보가 표시됩니다.</p>
      </div>
    {/if}
  {:else}
    <div class="detail-header">
      <div class="header-info">
        {#if $periodReturns !== null}
          <h2 class="etf-name">{$periodReturns.name}</h2>
        {:else}
          <h2 class="etf-name">{$selectedTicker}</h2>
        {/if}
        <span class="etf-ticker">{$selectedTicker}</span>
      </div>
      <button class="close-btn" onclick={onClearSelection} title="선택 해제">✕</button>
    </div>

    <!-- 기간별 등락률 -->
    <section class="section">
      <h3 class="section-title">📈 기간별 등락률</h3>
      {#if $periodReturnsLoading}
        <div class="loading-state">
          <p>⏳ 기간별 등락률을 불러오는 중...</p>
        </div>
      {:else if $periodReturnsError !== null}
        <div class="error-state">
          <p>❌ 조회 실패: {$periodReturnsError}</p>
        </div>
      {:else if $periodReturns !== null}
        <div class="returns-table-wrap">
          <table class="returns-table">
            <thead>
              <tr>
                <th class="period-col">기간</th>
                <th class="num-col">등락률</th>
                <th class="num-col">시작가</th>
                <th class="num-col">종료가</th>
              </tr>
            </thead>
            <tbody>
              {#each $periodReturns.returns as r (r.period)}
                <tr>
                  <td class="period-cell">{r.label}</td>
                  <td class="num-cell" style="color: {colorForChange(r.return_pct)}; font-weight: 600;">
                    {formatPct(r.return_pct)}
                  </td>
                  <td class="num-cell">{formatPrice(r.start_price)}</td>
                  <td class="num-cell">{formatPrice(r.end_price)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {:else}
        <div class="loading-state">
          <p>데이터를 불러오지 못했습니다.</p>
        </div>
      {/if}
    </section>

    <!-- 현재가 / 거래량 -->
    {#if $periodReturns !== null}
      <section class="section">
        <h3 class="section-title">💰 시세 요약</h3>
        <div class="summary-grid">
          <div class="summary-item">
            <span class="summary-label">현재가</span>
            <span class="summary-value">{formatPrice($periodReturns.current_price)}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">거래량</span>
            <span class="summary-value">{formatPrice($periodReturns.volume)}</span>
          </div>
        </div>
      </section>
    {/if}

    <!-- 가상 포지션 입력 -->
    <section class="section">
      <h3 class="section-title">📊 가상 포지션</h3>
      <p class="section-hint">참고용 가상 포지션 (실제 계좌 미연동)</p>
      <div class="position-form">
        <div class="form-row">
          <label class="form-label" for="buy-date">매수일</label>
          <input
            id="buy-date"
            type="date"
            class="form-input"
            bind:value={buyDate}
          />
        </div>
        <div class="form-row">
          <label class="form-label" for="avg-price">평균단가</label>
          <input
            id="avg-price"
            type="number"
            class="form-input num-input"
            placeholder="단가 입력"
            bind:value={avgPrice}
          />
        </div>
        <div class="form-row">
          <label class="form-label" for="qty">수량</label>
          <input
            id="qty"
            type="number"
            class="form-input num-input"
            placeholder="수량 입력"
            bind:value={qty}
          />
        </div>
        <div class="eval-row">
          <div class="eval-item">
            <span class="eval-label">평가손익</span>
            <span class="eval-value" style="color: {colorForChangeNullable(evalProfit)}">
              {formatNum(evalProfit)}
            </span>
          </div>
          <div class="eval-item">
            <span class="eval-label">수익률</span>
            <span class="eval-value" style="color: {colorForChangeNullable(evalPct)}">
              {formatPctNullable(evalPct)}
            </span>
          </div>
        </div>
        <button
          class="save-btn"
          onclick={onSavePosition}
          disabled={saving}
        >
          {#if saving}
            저장 중...
          {:else if saveStatus === 'done'}
            ✅ 저장 완료
          {:else if saveStatus === 'error'}
            ❌ 저장 실패
          {:else}
            💾 포지션 저장
          {/if}
        </button>
      </div>
    </section>
  {/if}
</div>

<style>
  .detail-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow-y: auto;
    padding-right: 4px;
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
  .detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
    padding: 10px 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
  }
  .header-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .etf-name {
    margin: 0;
    font-size: calc(1.05rem * var(--font-scale));
    color: var(--text);
    font-weight: 600;
  }
  .etf-ticker {
    font-size: calc(0.82rem * var(--font-scale));
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    font-size: calc(1rem * var(--font-scale));
    padding: 2px 6px;
    border-radius: 4px;
  }
  .close-btn:hover {
    color: var(--danger);
    background: var(--remove-hover-bg);
  }
  .section {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .section-title {
    margin: 0;
    font-size: calc(0.95rem * var(--font-scale));
    color: var(--text);
    font-weight: 600;
  }
  .section-hint {
    margin: -4px 0 0 0;
    font-size: calc(0.75rem * var(--font-scale));
    color: var(--text-muted);
    font-style: italic;
  }
  .loading-state,
  .error-state {
    padding: 20px;
    text-align: center;
    color: var(--text-dim);
    font-size: calc(0.88rem * var(--font-scale));
  }
  .error-state {
    color: var(--danger);
  }
  .returns-table-wrap {
    border: 1px solid var(--border-soft);
    border-radius: 6px;
    overflow: hidden;
  }
  .returns-table {
    width: 100%;
    border-collapse: collapse;
    font-size: calc(0.86rem * var(--font-scale));
  }
  .returns-table thead {
    background: var(--surface-2);
  }
  .returns-table th {
    text-align: left;
    padding: 7px 10px;
    color: var(--text-muted);
    font-weight: 600;
    border-bottom: 1px solid var(--border);
  }
  .returns-table th.num-col {
    text-align: right;
  }
  .returns-table td {
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-soft);
    color: var(--text);
  }
  .returns-table td.num-cell {
    text-align: right;
    font-family: var(--font-mono);
    font-size: calc(0.83rem * var(--font-scale));
  }
  .returns-table tbody tr:last-child td {
    border-bottom: none;
  }
  .summary-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .summary-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 10px;
    background: var(--surface-2);
    border-radius: 6px;
  }
  .summary-label {
    font-size: calc(0.75rem * var(--font-scale));
    color: var(--text-muted);
  }
  .summary-value {
    font-size: calc(1.05rem * var(--font-scale));
    color: var(--text);
    font-family: var(--font-mono);
    font-weight: 600;
  }
  .position-form {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .form-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .form-label {
    flex: 0 0 80px;
    font-size: calc(0.85rem * var(--font-scale));
    color: var(--text-secondary);
  }
  .form-input {
    flex: 1;
    background: var(--surface-input);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 6px 10px;
    font-size: calc(0.88rem * var(--font-scale));
    font-family: var(--font-mono);
  }
  .form-input:focus {
    border-color: var(--accent);
    outline: none;
  }
  .num-input {
    text-align: right;
  }
  .eval-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    padding: 8px 10px;
    background: var(--surface-2);
    border-radius: 6px;
  }
  .eval-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .eval-label {
    font-size: calc(0.75rem * var(--font-scale));
    color: var(--text-muted);
  }
  .eval-value {
    font-size: calc(1rem * var(--font-scale));
    font-family: var(--font-mono);
    font-weight: 600;
  }
  .save-btn {
    background: var(--accent);
    color: var(--text);
    border: none;
    border-radius: 6px;
    padding: 8px 16px;
    cursor: pointer;
    font-size: calc(0.9rem * var(--font-scale));
    font-weight: 600;
  }
  .save-btn:hover:not(:disabled) {
    background: var(--accent-hover);
  }
  .save-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* === 포트폴리오 요약 뷰 === */
  .portfolio-section {
    flex: 1;
    overflow-y: auto;
  }
  .summary-table-wrap {
    border: 1px solid var(--border-soft);
    border-radius: 6px;
    overflow: auto;
  }
  .summary-table {
    width: 100%;
    border-collapse: collapse;
    font-size: calc(0.86rem * var(--font-scale));
  }
  .summary-table thead {
    position: sticky;
    top: 0;
    background: var(--surface-2);
    z-index: 1;
  }
  .summary-table th {
    text-align: left;
    padding: 7px 10px;
    color: var(--text-muted);
    font-weight: 600;
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
  }
  .summary-table th.num-col {
    text-align: right;
  }
  .summary-table td {
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-soft);
    color: var(--text);
  }
  .summary-table td.num-cell {
    text-align: right;
    font-family: var(--font-mono);
    font-size: calc(0.83rem * var(--font-scale));
  }
  .summary-table .name-cell {
    font-weight: 500;
  }
  .summary-table .date-cell {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: calc(0.8rem * var(--font-scale));
  }
  .summary-row {
    cursor: pointer;
  }
  .summary-row:hover {
    background: var(--row-hover);
  }
  .portfolio-summary-bar {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    padding: 10px;
    background: var(--surface-2);
    border-radius: 6px;
    margin-top: 4px;
  }
</style>