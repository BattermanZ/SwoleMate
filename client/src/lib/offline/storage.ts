type KvEntry<T> = { key: string; value: T };

const DB_NAME = 'swolemate';
const DB_VERSION = 1;
const STORE = 'kv';

function openDb(): Promise<IDBDatabase> {
	return new Promise((resolve, reject) => {
		const request = indexedDB.open(DB_NAME, DB_VERSION);
		request.onupgradeneeded = () => {
			const db = request.result;
			if (!db.objectStoreNames.contains(STORE)) {
				db.createObjectStore(STORE, { keyPath: 'key' });
			}
		};
		request.onsuccess = () => resolve(request.result);
		request.onerror = () => reject(request.error ?? new Error('Failed to open IndexedDB'));
	});
}

async function withStore<R>(
	mode: IDBTransactionMode,
	fn: (store: IDBObjectStore) => IDBRequest<R>
): Promise<R> {
	const db = await openDb();
	return new Promise((resolve, reject) => {
		const tx = db.transaction(STORE, mode);
		const store = tx.objectStore(STORE);
		const request = fn(store);
		request.onsuccess = () => resolve(request.result);
		request.onerror = () => reject(request.error ?? new Error('IndexedDB request failed'));
	});
}

function supportsIdb(): boolean {
	return typeof window !== 'undefined' && typeof indexedDB !== 'undefined';
}

function lsKey(key: string): string {
	return `swolemate:${key}`;
}

export async function kvGet<T>(key: string): Promise<T | null> {
	if (typeof window === 'undefined') return null;

	if (!supportsIdb()) {
		const raw = localStorage.getItem(lsKey(key));
		if (!raw) return null;
		return JSON.parse(raw) as T;
	}

	const entry = await withStore<KvEntry<T> | undefined>('readonly', (store) => store.get(key));
	return entry?.value ?? null;
}

export async function kvSet<T>(key: string, value: T): Promise<void> {
	if (typeof window === 'undefined') return;

	if (!supportsIdb()) {
		localStorage.setItem(lsKey(key), JSON.stringify(value));
		return;
	}

	await withStore('readwrite', (store) => store.put({ key, value } satisfies KvEntry<T>));
}

export async function kvDelete(key: string): Promise<void> {
	if (typeof window === 'undefined') return;

	if (!supportsIdb()) {
		localStorage.removeItem(lsKey(key));
		return;
	}

	await withStore('readwrite', (store) => store.delete(key));
}

export async function kvListKeys(prefix: string): Promise<string[]> {
	if (typeof window === 'undefined') return [];

	if (!supportsIdb()) {
		const keys: string[] = [];
		for (let i = 0; i < localStorage.length; i++) {
			const k = localStorage.key(i);
			if (!k) continue;
			if (!k.startsWith('swolemate:')) continue;
			const plain = k.slice('swolemate:'.length);
			if (plain.startsWith(prefix)) keys.push(plain);
		}
		return keys.sort((a, b) => a.localeCompare(b));
	}

	const db = await openDb();
	return new Promise((resolve, reject) => {
		const tx = db.transaction(STORE, 'readonly');
		const store = tx.objectStore(STORE);
		const request = store.getAllKeys();
		request.onsuccess = () => {
			const keys = (request.result as string[]).filter((k) => k.startsWith(prefix));
			resolve(keys.sort((a, b) => a.localeCompare(b)));
		};
		request.onerror = () => reject(request.error ?? new Error('Failed to list keys'));
	});
}
