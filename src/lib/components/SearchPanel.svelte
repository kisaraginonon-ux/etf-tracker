// SearchPanel — ETF 종목 검색 및 즐겨찾기 추가 (REQ-F-14)

<script lang="ts">
  import { searchQuery, searchResults, performSearch, addFavoriteAction, favorites } from '$lib/stores';
  import type { EtfMasterItem } from '$lib/types';

  let localQuery = $state('');

  async function onSearch() {
    searchQuery.set(localQuery);
    await performSearch(localQuery);
  }

  async function onAdd(item: EtfMasterItem) {
    await addFavoriteAction(item);
  }

  function isFavorite(ticker: string): boolean {
    return $favorites.some(f => f.ticker === ticker);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') onSearch();
  }
</script>

<div class="search-panel">
  <div class="search-bar">
    <input
      type="text"
      placeholder="종목명 또는 코드 검색..."
      bind:value={localQuery}
      onkeydown={onKeydown}
      class="search-input"
    />
    <button onclick={onSearch} class="search-btn">검색</button>
  </div>

  {#if $searchResults.length > 0}
    <div class="results">
      {#each $searchResults as item (item.ticker)}
        <div class="result-row">
          <div class="item-info">
            <span class="ticker">{item.ticker}</span>
            <span class="name">{item.name}</span>
          </div>
          {#if isFavorite(item.ticker)}
            <span class="added-tag">✓ 추가됨</span>
          {:else}
            <button class="add-btn" onclick={() => onAdd(item)}>+ 추가</button>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .search-panel { margin-bottom: 16px; }
  .search-bar { display: flex; gap: 8px; }
  .search-input {
    flex: 1; background: var(--surface-input); color: var(--text);
    border: 1px solid var(--border-strong);
    border-radius: 8px; padding: 8px 12px; font-size: calc(0.9rem * var(--font-scale));
  }
  .search-input:focus { border-color: var(--accent); outline: none; }
  .search-btn {
    background: var(--accent); color: #fff; border: none; border-radius: 8px;
    padding: 8px 16px; cursor: pointer;
    font-size: calc(0.9rem * var(--font-scale)); font-weight: 600;
  }
  .search-btn:hover { background: var(--accent-hover); }
  .results {
    margin-top: 8px; max-height: 240px; overflow-y: auto;
    background: var(--surface); border: 1px solid var(--border); border-radius: 8px;
  }
  .result-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 12px; border-bottom: 1px solid var(--border-soft);
  }
  .result-row:last-child { border-bottom: none; }
  .item-info { display: flex; gap: 12px; align-items: center; }
  .ticker {
    color: var(--text-muted); font-size: calc(0.85rem * var(--font-scale));
    font-family: var(--font-mono); min-width: 70px;
  }
  .name { color: var(--text); font-size: calc(0.9rem * var(--font-scale)); }
  .added-tag {
    color: var(--success);
    font-size: calc(0.85rem * var(--font-scale));
  }
  .add-btn {
    background: var(--accent-bg); color: var(--accent);
    border: 1px solid var(--accent-border);
    border-radius: 6px; padding: 4px 12px; cursor: pointer;
    font-size: calc(0.85rem * var(--font-scale));
  }
  .add-btn:hover { background: var(--accent); color: #fff; }
</style>