import { writable } from 'svelte/store';

export type Theme = 'light' | 'dark';

const STORAGE_KEY = 'dept-tracker-theme';

function readStoredTheme(): Theme {
  if (typeof localStorage === 'undefined') return 'light';
  return localStorage.getItem(STORAGE_KEY) === 'dark' ? 'dark' : 'light';
}

export function applyTheme(theme: Theme) {
  if (typeof document === 'undefined') return;
  document.documentElement.setAttribute('data-theme', theme);
  const meta = document.querySelector('meta[name="theme-color"]');
  meta?.setAttribute('content', theme === 'dark' ? '#0b1219' : '#f0f4f8');
}

export const theme = writable<Theme>(readStoredTheme());

theme.subscribe((value) => {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, value);
  }
  applyTheme(value);
});

if (typeof document !== 'undefined') {
  applyTheme(readStoredTheme());
}

export function toggleTheme() {
  theme.update((current) => (current === 'light' ? 'dark' : 'light'));
}
