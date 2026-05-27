/** Toggle light/dark theme on <html>, persisting the choice to localStorage. */
export function toggleTheme(): void {
	const root = document.documentElement;
	const isDark = root.getAttribute('data-theme') === 'dark';
	const next = isDark ? 'light' : 'dark';
	if (next === 'dark') {
		root.setAttribute('data-theme', 'dark');
		root.classList.add('dark');
	} else {
		root.removeAttribute('data-theme');
		root.classList.remove('dark');
	}
	try {
		localStorage.setItem('theme', next);
	} catch {
		/* ignore */
	}
}
