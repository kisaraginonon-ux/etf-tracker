// DisclaimerModal — 면책 조항 모달 (REQ-NF-05)
// 앱 실행 시 1회 안내 메시지 표시, settings 테이블에 저장

<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  let show = $state(false);

  onMount(async () => {
    try {
      const acknowledged = await invoke<string>('get_setting', { key: 'disclaimer_acknowledged' });
      if (acknowledged !== 'true') {
        show = true;
      }
    } catch (e) {
      // 백엔드 명령이 없으면 무조건 표시
      show = true;
    }
  });

  async function handleAcknowledge() {
    try {
      await invoke('set_setting', { key: 'disclaimer_acknowledged', value: 'true' });
    } catch (e) {
      console.error('Failed to save disclaimer ack:', e);
    }
    show = false;
  }
</script>

{#if show}
  <div class="modal-overlay" onclick={handleAcknowledge}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <h2 class="modal-title">📋 면책 조항</h2>
      <div class="modal-body">
        <p>본 애플리케이션은 <strong>초개인 투자자</strong>를 위한 ETF 데이터 모니터링 도구입니다.</p>
        <ul>
          <li>📊 본 데이터는 <strong>투자 참고용</strong>이며 오류 가능성이 있습니다.</li>
          <li>⚠️ 본 앱의 데이터를 기반으로 한 투자 결정의 책임은 전적으로 사용자에게 있습니다.</li>
          <li>📈 가상 포지션은 <strong>참고용</strong>이며 실제 계좌와 연동되지 않습니다.</li>
          <li>🔔 알림은 데이터 지연, Provider 장애 등으로 누락될 수 있습니다.</li>
          <li>💰 데이터 소스(Naver/Yahoo)는 실시간이 아닌 수초~수분 지연될 수 있습니다.</li>
        </ul>
        <p class="modal-footer-text">확인 버튼을 누르면 다시 표시되지 않습니다.</p>
      </div>
      <button class="modal-btn" onclick={handleAcknowledge}>확인</button>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
  }
  .modal-content {
    background: #1e1e2e;
    border: 1px solid #444;
    border-radius: 12px;
    padding: 28px 32px;
    max-width: 480px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
  .modal-title {
    font-size: 1.2rem;
    color: #e0e0e0;
    margin: 0 0 16px 0;
  }
  .modal-body {
    font-size: 0.9rem;
    color: #bbb;
    line-height: 1.6;
  }
  .modal-body p { margin: 0 0 12px 0; }
  .modal-body ul {
    margin: 0 0 12px 0;
    padding-left: 20px;
  }
  .modal-body li { margin-bottom: 6px; }
  .modal-body strong { color: #e0e0e0; }
  .modal-footer-text {
    font-size: 0.8rem;
    color: #666;
    margin-top: 12px;
  }
  .modal-btn {
    margin-top: 20px;
    width: 100%;
    padding: 10px;
    background: #6366f1;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
  }
  .modal-btn:hover { background: #5457e0; }
</style>