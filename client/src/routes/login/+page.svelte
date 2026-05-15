<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth } from '$lib/auth';
	import { logger } from '$lib/logger';
	import { get } from 'svelte/store';
	import { Btn, Card } from '$lib/components/ui';

	let username = $state('');
	let password = $state('');
	let loading = $state(false);
	let error = $state<string | null>(null);

	async function submit() {
		error = null;
		loading = true;
		try {
			await auth.login(username, password);
			logger.info('auth', 'Logged in');
			const mustChange = get(auth.state).user?.must_change_password ?? false;
			await goto(mustChange ? '/settings' : '/');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Login failed';
		} finally {
			loading = false;
		}
	}
</script>

<main class="login">
	<header>
		<div class="logo">SM</div>
		<h1>Welcome <em>back.</em></h1>
		<p>Sign in with your SwoleMate username and password.</p>
	</header>

	<Card>
		<form
			onsubmit={(e) => {
				e.preventDefault();
				void submit();
			}}
		>
			<label>
				<span class="lbl">Username</span>
				<input
					bind:value={username}
					autocomplete="username"
					inputmode="text"
					disabled={loading}
					required
				/>
			</label>
			<label>
				<span class="lbl">Password</span>
				<input
					type="password"
					bind:value={password}
					autocomplete="current-password"
					disabled={loading}
					required
				/>
			</label>
			{#if error}<div class="err">{error}</div>{/if}
			<Btn variant="primary" type="submit" disabled={loading}>
				{loading ? 'Signing in…' : 'Sign in'}
			</Btn>
		</form>
		<p class="hint">
			When offline, you can still log locally; sign in again when you're back online to sync.
		</p>
	</Card>
</main>

<style>
	.login {
		min-height: 100dvh;
		display: grid;
		place-items: center;
		padding: 32px 18px;
	}
	header {
		text-align: center;
		margin-bottom: 18px;
		max-width: 420px;
	}
	.logo {
		width: 40px;
		height: 40px;
		border-radius: 12px;
		background: linear-gradient(135deg, var(--clay-2), var(--clay));
		color: white;
		display: grid;
		place-items: center;
		font: 800 16px/1 'Onest', system-ui, sans-serif;
		box-shadow: 0 6px 14px -4px rgba(255, 94, 31, 0.55);
		margin: 0 auto 14px;
	}
	h1 {
		margin: 0;
		font: 800 32px/1 'Onest', system-ui, sans-serif;
		letter-spacing: -0.025em;
	}
	h1 em {
		font: italic 400 32px/1 'Instrument Serif';
		color: var(--clay-text);
	}
	p {
		margin: 8px 0 0;
		font: 500 14px/1.4 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
	}
	form {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	form :global(button) {
		width: 100%;
	}
	label {
		display: block;
	}
	.lbl {
		display: block;
		font: 700 10px/1 'Onest', system-ui, sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
		margin-bottom: 6px;
	}
	input {
		width: 100%;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 10px;
		padding: 12px 14px;
		font: 500 14px/1.2 'Onest', system-ui, sans-serif;
		color: var(--ink);
		outline: 0;
	}
	input:focus {
		border-color: var(--clay);
		box-shadow: 0 0 0 3px rgba(255, 94, 31, 0.16);
	}
	.err {
		font: 600 13px/1.4 'Onest', system-ui, sans-serif;
		color: var(--clay-text);
	}
	.hint {
		margin: 12px 0 0;
		font: italic 400 12px/1.4 'Instrument Serif';
		color: var(--ink-soft);
		text-align: center;
	}

	.login :global(.card) {
		max-width: 420px;
		width: 100%;
	}
</style>
