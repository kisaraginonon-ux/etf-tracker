// AlertSettings — 가격 알림 설정 (REQ-F-11, REQ-F-12)
// 목표가(상한) / 손절가(하한) 임계치 설정

<script lang="ts">
  import { alerts, loadAlerts, setAlertAction, resetAlertAction, removeAlertAction, favorites } from '$lib/stores';
  import type { AlertType } from '$lib/types';

  let editingTicker = $state<string | null>(null);
  let editType = $state<AlertType>('target');
  let threshold = $state('');

  // 즐겨찾기 기준 알림 목록
  let alertRows = $derived(
    $favorites.map(fav => {
      const target = $alerts.find(a => a.ticker === fav.ticker && a.alert_type === 'target');
      const stop = $alerts.find(a => a.ticker === fav.ticker && a.alert_type === 'stop_loss');
      return { ticker: fav.ticker, name: fav.name, target, stop };
    })
  );

  function startSet(ticker: string, type: AlertType, current: number | undefined) {
    editingTicker = ticker;
    editType = type;
    threshold = current ? current.toString() : '';
  }

  function cancelEdit() {
    editingTicker = null;
    threshold = '';
  }

  async function saveAlert() {
    if (!editingTicker) return;
    const t = parseFloat(threshold);
    if (isNaN(t) || t <= 0) return;
    await setAlertAction(editingTicker, editType, t);
    cancelEdit();
  }

  async function onReset(ticker: string) {
    await resetAlertAction(ticker);
  }

  async function onRemove(ticker: string, type: AlertType) {
    await removeAlertAction(ticker, type);
  }
</script>

<div class="alert-panel">
  <div class="header">
    <h3>가격 알림</h3>
    <span class="notice">목표가/손절가 도달 시 1회 알림 (토스트)</span>
  </div>

  {#if editingTicker}
    <div class="edit-form">
      <h4>{editingTicker} — {editType === 'target' ? '목표가 설정' : '손절가 설정'}</h4>
      <div class="form-row">
        <label for="threshold-input">임계가 (원)</label>
        <input id="threshold-input" type="number" placeholder="예: 140000" bind:value={threshold} />
      </div>
      <div class="form-actions">
        <button class="btn-save" onclick={saveAlert}>저장</button>
        <button class="btn-cancel" onclick={cancelEdit}>취소</button>
      </div>
    </div>
  {/if}

  {#if alertRows.length > 0}
    <table class="alert-table">
      <thead>
        <tr>
          <th>종목명</th>
          <th>목표가</th>
          <th>손절가</th>
          <th>관리</th>
        </tr>
      </thead>
      <tbody>
        {#each alertRows as row (row.ticker)}
          <tr>
            <td class="name-cell">{row.name}</td>
            <td class="num-cell">
              {#if row.target}
                <span class="alert-tag target" onclick={() => onReset(row.ticker)}>
                  {row.target.threshold.toLocaleString('ko-KR')}원 ⟳
                </span>
              {:else}
                <button class="btn-set" onclick={() => startSet(row.ticker, 'target', undefined)}>+ 목표가</button>
              {/if}
            </td>
            <td class="num-cell">
              {#if row.stop}
                <span class="alert-tag stop" onclick={() => onReset(row.ticker)}>
                  {row.stop.threshold.toLocaleString('ko-KR')}원 ⟳
                </span>
              {:else}
                <button class="btn-set stop" onclick={() => startSet(row.ticker, 'stop_loss', undefined)}>+ 손절가</button>
              {/if}
            </td>
            <td class="action-cell">
              {#if row.target}
                <button class="btn-del" onclick={() => onRemove(row.ticker, 'target')}>🗑️T</button>
              {/if}
              {#if row.stop}
                <button class="btn-del" onclick={() => onRemove(row.ticker, 'stop_loss')}>🗑️S</button>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <p class="empty">즐겨찾기에 종목을 추가하면 알림을 설정할 수 있습니다.</p>
  {/if}
</div>

<style>
  .alert-panel { background: #1e1e2e; border: 1px solid #333; border-radius: 8px; padding: 16px; margin-bottom: 16px; }
  .header { display: flex; align-items: center; gap: 12px; margin-bottom: 12px; }
  .header h3 { margin: 0; font-size: 1rem; color: #e0e0e0; }
  .notice { font-size: 0.75rem; color: #888; font-style: italic; }
  .edit-form { background: #252535; border-radius: 8px; padding: 12px; margin-bottom: 12px; }
  .edit-form h4 { margin: 0 0 10px; font-size: 0.9rem; color: #6366f1; }
  .form-row { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
  .form-row label { width: 80px; font-size: 0.85rem; color: #aaa; }
  .form-row input { flex: 1; background: #1a1a2a; color: #e0e0e0; border: 1px solid #444; border-radius: 6px; padding: 6px 10px; font-size: 0.85rem; }
  .form-actions { display: flex; gap: 8px; margin-top: 10px; }
  .btn-save { background: #6366f1; color: #fff; border: none; border-radius: 6px; padding: 6px 16px; cursor: pointer; font-size: 0.85rem; }
  .btn-save:hover { background: #5457e5; }
  .btn-cancel { background: #333; color: #ccc; border: 1px solid #444; border-radius: 6px; padding: 6px 16px; cursor: pointer; font-size: 0.85rem; }
  .alert-table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
  .alert-table th { text-align: left; padding: 6px 8px; color: #888; border-bottom: 1px solid #333; font-weight: 600; }
  .alert-table td { padding: 6px 8px; border-bottom: 1px solid #252535; color: #e0e0e0; }
  .num-cell { text-align: center; }
  .name-cell { font-weight: 500; }
  .action-cell { text-align: center; white-space: nowrap; }
  .alert-tag { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 0.8rem; cursor: pointer; font-family: monospace; }
  .alert-tag.target { background: rgba(239,83,80,0.15); color: #ef5350; }
  .alert-tag.stop { background: rgba(38,166,154,0.15); color: #26a69a; }
  .btn-set { background: #2a2a4a; color: #6366f1; border: 1px solid #444; border-radius: 4px; padding: 2px 8px; cursor: pointer; font-size: 0.8rem; }
  .btn-set.stop { color: #26a69a; }
  .btn-set:hover { background: #3a3a5a; }
  .btn-del { background: transparent; border: none; cursor: pointer; padding: 2px 4px; font-size: 0.8rem; opacity: 0.6; }
  .btn-del:hover { opacity: 1; }
  .empty { color: #666; font-size: 0.85rem; text-align: center; padding: 20px; }
</style>