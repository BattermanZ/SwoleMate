import { writable } from 'svelte/store';
import { syncOfflineMutations } from '$lib/api';

const isBrowser = typeof window !== 'undefined';
const initial = isBrowser ? navigator.onLine : true;

export const online = writable(initial);

if (isBrowser) {
    const handleOnline = () => {
        online.set(true);
        void syncOfflineMutations();
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', () => online.set(false));

    if (navigator.onLine) {
        void syncOfflineMutations();
    }
}
