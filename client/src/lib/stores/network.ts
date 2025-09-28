import { writable } from 'svelte/store';

const isBrowser = typeof window !== 'undefined';
const initial = isBrowser ? navigator.onLine : true;

export const online = writable(initial);

if (isBrowser) {
    window.addEventListener('online', () => online.set(true));
    window.addEventListener('offline', () => online.set(false));
}
