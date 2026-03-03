const CACHE_PREFIX = 'swolemate-cache:';

const isBrowser = typeof window !== 'undefined';

type CachedValue<T> = {
    timestamp: number;
    data: T;
};

export function saveToCache<T>(key: string, data: T): void {
    if (!isBrowser) return;
    try {
        const payload: CachedValue<T> = {
            timestamp: Date.now(),
            data
        };
        localStorage.setItem(`${CACHE_PREFIX}${key}`, JSON.stringify(payload));
    } catch (error) {
        console.error('Failed to save to offline cache', error);
    }
}

export function getFromCache<T>(key: string): T | null {
    if (!isBrowser) return null;
    try {
        const value = localStorage.getItem(`${CACHE_PREFIX}${key}`);
        if (!value) return null;
        const parsed = JSON.parse(value) as CachedValue<T>;
        return parsed.data;
    } catch (error) {
        console.error('Failed to read from offline cache', error);
        return null;
    }
}

export function clearCache(key: string): void {
    if (!isBrowser) return;
    localStorage.removeItem(`${CACHE_PREFIX}${key}`);
}

export function isOnline(): boolean {
    if (!isBrowser) return true;
    return navigator.onLine;
}
