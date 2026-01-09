<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth } from '$lib/auth';
	import { logger } from '$lib/logger';
	import { get } from 'svelte/store';

	let username = '';
	let password = '';
	let loading = false;
	let error: string | null = null;

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

<div class="mx-auto w-full max-w-md space-y-4">
	<header class="space-y-1 text-center">
		<h1 class="text-3xl font-black tracking-tight">Sign in</h1>
		<p class="text-sm opacity-75">Use your SwoleMate username and password.</p>
	</header>

	<div class="card variant-glass-surface p-4 space-y-4">
		<form
			class="space-y-4"
			on:submit|preventDefault={() => {
				void submit();
			}}
		>
			<label class="space-y-1 block">
				<span class="text-sm font-semibold">Username</span>
				<input
					class="input w-full"
					autocomplete="username"
					inputmode="text"
					bind:value={username}
					disabled={loading}
				/>
			</label>

			<label class="space-y-1 block">
				<span class="text-sm font-semibold">Password</span>
				<input
					type="password"
					class="input w-full"
					autocomplete="current-password"
					bind:value={password}
					disabled={loading}
				/>
			</label>

			{#if error}
				<div class="text-sm text-error-500">{error}</div>
			{/if}

			<button type="submit" class="btn variant-filled-primary w-full" disabled={loading}>
				{loading ? 'Signing in…' : 'Sign in'}
			</button>
		</form>

		<div class="text-xs opacity-70">
			When offline, you can still log locally; sign in again when you’re back online to sync.
		</div>
	</div>
</div>
