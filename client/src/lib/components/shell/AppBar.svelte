<script lang="ts">
	import { Logo } from '$lib/components/ui';
	interface Props {
		onLogout?: () => void;
	}
	let { onLogout }: Props = $props();

	function toggleTheme() {
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
</script>

<header class="appbar">
	<a class="brand" href="/" aria-label="SwoleMate home">
		<Logo size={30} />
		<span class="name">SwoleMate</span>
	</a>
	<div class="actions">
		<button
			type="button"
			class="icon-btn theme-toggle"
			aria-label="Toggle dark mode"
			onclick={toggleTheme}
		>
			<svg
				class="ico-moon"
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
			</svg>
			<svg
				class="ico-sun"
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<circle cx="12" cy="12" r="4" />
				<path
					d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"
				/>
			</svg>
		</button>
		{#if onLogout}
			<button type="button" class="icon-btn" aria-label="Log out" onclick={onLogout}>
				<svg
					width="18"
					height="18"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
					<polyline points="16 17 21 12 16 7" />
					<line x1="21" y1="12" x2="9" y2="12" />
				</svg>
			</button>
		{/if}
	</div>
</header>

<style>
	.appbar {
		position: sticky;
		top: 0;
		z-index: 20;
		padding: calc(12px + env(safe-area-inset-top)) 16px 10px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		background: color-mix(in oklab, var(--bg) 82%, transparent);
		backdrop-filter: blur(12px);
		border-bottom: 1px solid var(--line);
		gap: 8px;
	}
	.brand {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
		text-decoration: none;
		color: inherit;
	}
	.name {
		font:
			800 17px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.01em;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.actions {
		display: flex;
		align-items: center;
		gap: 2px;
		flex: none;
	}
	.icon-btn {
		width: 34px;
		height: 34px;
		border-radius: 999px;
		background: transparent;
		border: 0;
		color: var(--ink-2);
		display: grid;
		place-items: center;
		cursor: pointer;
	}
	.icon-btn:active {
		background: var(--bg-2);
	}

	.theme-toggle .ico-sun {
		display: none;
	}
	.theme-toggle .ico-moon {
		display: inline-block;
	}
	:global([data-theme='dark']) .theme-toggle .ico-sun {
		display: inline-block;
	}
	:global([data-theme='dark']) .theme-toggle .ico-moon {
		display: none;
	}
</style>
