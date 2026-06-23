// PositionPanel — 가상 포지션 입력/수정 (REQ-F-07, REQ-F-08)
// "참고용 가상 포지션 (실제 계좌 미연동)" 명시

<script lang="ts">
  import { positions, loadPositions, setPositionAction, removePositionAction, favorites } from '$lib/stores';
  import type { VirtualPosition } from '$lib/types';

  let editingTicker = $state<string | null>(null);
  let buyDate = $state('');
  let avgPrice = $state('');
  let quantity = $state('');

  // 즐겨찾기 중 포지션 없는 종목 표시
  let favoritesWithoutPosition = $derived(
    $favorites.filter(f => !$positions.some(p => p.ticker === f.ticker))
  );

  // 포지션 있는 종목 매핑
  let positionRows = $derived(
    $positions.map(p => {
      const fav = $favorites.find(f => f.ticker === p.ticker);
      return { ...p, name: (fav && fav.name) ? fav.name : p.ticker };
    })
  );

  function startEdit(ticker: string, pos: VirtualPosition | undefined) {
    editingTicker = ticker;
    buyDate = (pos && pos.buy_date) || '';
    avgPrice = (pos && pos.avg_buy_price) ? pos.avg_buy_price.toString() : '';
    quantity = (pos && pos.quantity) ? pos.quantity.toString() : '';
  }

  function cancelEdit() {
    editingTicker = null;
    buyDate = '';
    avgPrice = '';
    quantity = '';
  }

  async function savePosition() {
    if (!editingTicker) return;
    await setPositionAction(
      editingTicker,
      buyDate || null,
      avgPrice ? parseFloat(avgPrice) : null,
      quantity ? parseFloat(quantity) : null,
    );
    cancelEdit();
  }

  async function deletePosition(ticker: string) {
    await removePositionAction(ticker);
  }

  // 평가손익 계산 (현재가 필요하지만 아직 API 연동 전이므로 0)
  function evalProfit(pos: VirtualPosition): number | null {
    if (!pos.avg_buy_price || !pos.quantity) return null;
    // TODO: current_price 연동 후 (current - avg) * qty
    return null;
  }
</script>

<div class="position-panel">
  <div class="header">
    <h3>가상 포지션</h3>
    <span class="notice">참고용 가상 포지션 (실제 계좌 미연동)</span>
  </div>

  {#if editingTicker}
    <div class="edit-form">
      <h4>{editingTicker} 포지션 {#if $positions.some(p => p.ticker === editingTicker)}수정{:else}추가{/if}</h4>
      <div class="form-row">
        <label>매수일</label>
        <input type="date" bind:value={buyDate} />
      </div>
      <div class="form-row">
        <label>평균 매수단가</label>
        <input type="number" placeholder="예: 134000" bind:value={avgPrice} />
      </div>
      <div class="form-row">
        <label>수량</label>
        <input type="number" placeholder="예: 10" bind:value={quantity} />
      </div>
      <div class="form-actions">
        <button class="btn-save" onclick={savePosition}>저장</button>
        <button class="btn-cancel" onclick={cancelEdit}>취소</button>
      </div>
    </div>
  {/if}

  {#if positionRows.length > 0}
    <table class="pos-table">
      <thead>
        <tr>
          <th>종목명</th>
          <th>매수일</th>
          <th>단가</th>
          <th>수량</th>
          <th>평가손익</th>
          <th>수익률</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each positionRows as row (row.ticker)}
          <tr>
            <td class="name-cell">{row.name}</td>
            <td>{row.buy_date || '-'}</td>
            <td class="num-cell">{row.avg_buy_price ? row.avg_buy_price.toLocaleString('ko-KR') : '-'}</td>
            <td class="num-cell">{row.quantity || '-'}</td>
            <td class="num-cell">{
              (() => {
                const p = evalProfit(row);
                return p !== null ? p.toLocaleString('ko-KR') : '-';
              })()
            }</td>
            <td class="num-cell">-</td>
            <td class="action-cell">
              <button class="btn-edit" onclick={() => startEdit(row.ticker, row as VirtualPosition)}>✏️</button>
              <button class="btn-del" onclick={() => deletePosition(row.ticker)}>🗑️</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <p class="empty">등록된 가상 포지션이 없습니다.</p>
  {/if}

  {#if favoritesWithoutPosition.length > 0 && !editingTicker}
    <div class="add-section">
      <h4>포지션 추가</h4>
      <div class="add-buttons">
        {#each favoritesWithoutPosition as fav (fav.ticker)}
          <button class="btn-add-pos" onclick={() => startEdit(fav.ticker, undefined)}>
            {fav.name} ({fav.ticker})
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .position-panel { background: #1e1e2e; border: 1px solid #333; border-radius: 8px; padding: 16px; margin-bottom: 16px; }
  .header { display: flex; align-items: center; gap: 12px; margin-bottom: 12px; }
  .header h3 { margin: 0; font-size: 1rem; color: #e0e0e0; }
  .notice { font-size: 0.75rem; color: #888; font-style: italic; }
  .edit-form { background: #252535; border-radius: 8px; padding: 12px; margin-bottom: 12px; }
  .edit-form h4 { margin: 0 0 10px; font-size: 0.9rem; color: #6366f1; }
  .form-row { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
  .form-row label { width: 90px; font-size: 0.85rem; color: #aaa; }
  .form-row input { flex: 1; background: #1a1a2a; color: #e0e0e0; border: 1px solid #444; border-radius: 6px; padding: 6px 10px; font-size: 0.85rem; }
  .form-actions { display: flex; gap: 8px; margin-top: 10px; }
  .btn-save { background: #6366f1; color: #fff; border: none; border-radius: 6px; padding: 6px 16px; cursor: pointer; font-size: 0.85rem; }
  .btn-save:hover { background: #5457e5; }
  .btn-cancel { background: #333; color: #ccc; border: 1px solid #444; border-radius: 6px; padding: 6px 16px; cursor: pointer; font-size: 0.85rem; }
  .pos-table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
  .pos-table th { text-align: left; padding: 6px 8px; color: #888; border-bottom: 1px solid #333; font-weight: 600; }
  .pos-table td { padding: 6px 8px; border-bottom: 1px solid #252535; color: #e0e0e0; }
  .num-cell { text-align: right; font-family: monospace; }
  .name-cell { font-weight: 500; }
  .action-cell { text-align: center; white-space: nowrap; }
  .btn-edit, .btn-del { background: transparent; border: none; cursor: pointer; padding: 2px 4px; font-size: 0.9rem; }
  .btn-edit:hover { filter: brightness(1.3); }
  .btn-del:hover { filter: brightness(1.3); }
  .empty { color: #666; font-size: 0.85rem; text-align: center; padding: 20px; }
  .add-section { margin-top: 12px; }
  .add-section h4 { font-size: 0.85rem; color: #888; margin: 0 0 8px; }
  .add-buttons { display: flex; flex-wrap: wrap; gap: 6px; }
  .btn-add-pos { background: #2a2a4a; color: #6366f1; border: 1px solid #444; border-radius: 6px; padding: 4px 10px; cursor: pointer; font-size: 0.8rem; }
  .btn-add-pos:hover { background: #6366f1; color: #fff; }
</style>