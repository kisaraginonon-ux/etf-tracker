// Theme store — 테마 및 글씨 크기 전역 상태
// settings 테이블의 'theme' / 'font_scale' 키에 저장
// CSS 변수는 document.documentElement의 data-theme / data-font-scale 속성으로 적용

import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export type ThemeName = 'dark' | 'light' | 'high-contrast';
export type FontScale = 'small' | 'normal' | 'large';

export const VALID_THEMES: ThemeName[] = ['dark', 'light', 'high-contrast'];
export const VALID_SCALES: FontScale[] = ['small', 'normal', 'large'];

function isThemeName(v: string): boolean {
  return VALID_THEMES.indexOf(v as ThemeName) !== -1;
}

function isFontScale(v: string): boolean {
  return VALID_SCALES.indexOf(v as FontScale) !== -1;
}

export const theme = writable<ThemeName>('dark');
export const fontScale = writable<FontScale>('normal');

/** document.documentElement에 data-theme 속성 적용 + localStorage 동기화 */
function applyThemeToDom(t: ThemeName): void {
  const el = document.documentElement;
  el.setAttribute('data-theme', t);
  try { localStorage.setItem('etf-theme', t); } catch (e) { /* ignore */ }
}

/** document.documentElement에 data-font-scale 속성 적용 + localStorage 동기화 */
function applyFontScaleToDom(s: FontScale): void {
  const el = document.documentElement;
  el.setAttribute('data-font-scale', s);
  try { localStorage.setItem('etf-font-scale', s); } catch (e) { /* ignore */ }
}

/** 테마 로드 (settings 테이블에서 읽어 DOM + store에 적용) */
export async function loadThemeSettings(): Promise<void> {
  // 1. 인라인 스크립트가 이미 초기값을 적용했을 수 있음 — 우선 DOM에서 읽기
  const domTheme = document.documentElement.getAttribute('data-theme');
  if (domTheme !== null && isThemeName(domTheme)) {
    theme.set(domTheme as ThemeName);
  } else {
    applyThemeToDom('dark');
  }

  const domScale = document.documentElement.getAttribute('data-font-scale');
  if (domScale !== null && isFontScale(domScale)) {
    fontScale.set(domScale as FontScale);
  } else {
    applyFontScaleToDom('normal');
  }

  // 2. 백엔드 settings에서 최신값 로드 (인라인 스크립트보다 정확한 경우 반영)
  try {
    const t = await invoke<string>('get_setting', { key: 'theme' });
    if (t !== null && t !== undefined && t !== '' && isThemeName(t)) {
      const tn = t as ThemeName;
      theme.set(tn);
      applyThemeToDom(tn);
    }
  } catch (e) {
    // 백엔드 호출 실패 시 현재 DOM 값 유지
    console.error('Failed to load theme setting:', e);
  }

  try {
    const s = await invoke<string>('get_setting', { key: 'font_scale' });
    if (s !== null && s !== undefined && s !== '' && isFontScale(s)) {
      const fs = s as FontScale;
      fontScale.set(fs);
      applyFontScaleToDom(fs);
    }
  } catch (e) {
    console.error('Failed to load font_scale setting:', e);
  }
}

/** 테마 변경 (DOM + store + settings 저장) */
export async function setThemeAction(t: ThemeName): Promise<void> {
  theme.set(t);
  applyThemeToDom(t);
  try {
    await invoke('set_setting', { key: 'theme', value: t });
  } catch (e) {
    console.error('Failed to save theme:', e);
  }
}

/** 글씨 크기 변경 (DOM + store + settings 저장) */
export async function setFontScaleAction(s: FontScale): Promise<void> {
  fontScale.set(s);
  applyFontScaleToDom(s);
  try {
    await invoke('set_setting', { key: 'font_scale', value: s });
  } catch (e) {
    console.error('Failed to save font_scale:', e);
  }
}