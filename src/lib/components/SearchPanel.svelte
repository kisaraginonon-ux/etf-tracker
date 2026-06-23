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
    flex: 1; background: #2a2a3a; color: #e0e0e0; border: 1px solid #444;
    border-radius: 8px; padding: 8px 12px; font-size: 0.9rem;
  }
  .search-input:focus { border-color: #6366f1; outline: none; }
  .search-btn {
    background: #6366f1; color: #fff; border: none; border-radius: 8px;
    padding: 8px 16px; cursor: pointer; font-size: 0.9rem; font-weight: 600;
  }
  .search-btn:hover { background: #5457e5; }
  .results {
    margin-top: 8px; max-height: 240px; overflow-y: auto;
    background: #1e1e2e; border: 1px solid #333; border-radius: 8px;
  }
  .result-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 12px; border-bottom: 1px solid #2a2a3a;
  }
  .result-row:last-child { border-bottom: none; }
  .item-info { display: flex; gap: 12px; align-items: center; }
  .ticker { color: #888; font-size: 0.85rem; font-family: monospace; min-width: 70px; }
  .name { color: #e0e0e0; font-size: 0.9rem; }
  .added-tag { color: #4caf50; font-size: 0.85rem; }
  .add-btn {
    background: #2a2a4a; color: #6366f1; border: 1px solid #6366f1;
    border-radius: 6px; padding: 4px 12px; cursor: pointer; font-size: 0.85rem;
  }
  .add-btn:hover { background: #6366f1; color: #fff; }
</style>