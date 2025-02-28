declare module '$lib/stores/navigationStore' {
  import { Writable } from 'svelte/store';
  
  export const navigationHistory: Writable<Array<{path: string, title: string}>>;
  export function addPath(path: string, title: string): void;
  export function clearHistory(): void;
  export function goBack(): Writable<Array<{path: string, title: string}>>;
} 