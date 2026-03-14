<script lang="ts">
	import { onMount } from 'svelte';
	import {
		createMcpToken,
		getMcpTokens,
		revokeMcpToken,
		rotateMcpToken,
		type McpTokenSummary
	} from '$lib/api';
	import { auth } from '$lib/auth';

	const authState = auth.state;
	let currentPassword = '';
	let newPassword = '';
	let confirmPassword = '';
	let accountLoading = false;
	let accountError: string | null = null;
	let accountNotice: string | null = null;
	let mcpTokens: McpTokenSummary[] = [];
	let activeMcpTokens: McpTokenSummary[] = [];
	let mcpLoading = false;
	let mcpError: string | null = null;
	let mcpNotice: string | null = null;
	let creatingMcpToken = false;
	let revokingTokenId: number | null = null;
	let rotatingTokenId: number | null = null;
	let newTokenName = '';
	let newTokenAccess: 'read' | 'write' = 'read';
	let newTokenExpiryDays = 30;
	let createdToken:
		| {
				name: string;
				token: string;
				scopes: string[];
				expires_at: string | null;
		  }
		| null = null;

	function mcpScopesForAccess(access: 'read' | 'write'): string[] {
		if (access === 'write') {
			return ['workouts.read', 'progress.read', 'workouts.write'];
		}
		return ['workouts.read', 'progress.read'];
	}

	function formatDateTime(value: string | null): string {
		if (!value) return 'Never';
		const date = new Date(value);
		if (!Number.isFinite(date.getTime())) return value;
		return date.toLocaleString([], {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function describeTokenAccess(scopes: string[]): string {
		return scopes.includes('workouts.write') ? 'Read and write' : 'Read only';
	}

	async function loadMcpTokens() {
		if ($authState.status !== 'authenticated' || !$authState.user || $authState.offline) {
			mcpTokens = [];
			return;
		}

		mcpLoading = true;
		mcpError = null;
		try {
			mcpTokens = await getMcpTokens();
		} catch (e) {
			mcpError = e instanceof Error ? e.message : 'Failed to load MCP tokens';
		} finally {
			mcpLoading = false;
		}
	}

	async function handleCreateMcpToken() {
		mcpError = null;
		mcpNotice = null;
		createdToken = null;
		if ($authState.status !== 'authenticated' || !$authState.user) {
			mcpError = 'Sign in to create MCP access tokens.';
			return;
		}
		if ($authState.offline) {
			mcpError = 'Offline mode: create MCP access tokens when online.';
			return;
		}
		if (!newTokenName.trim()) {
			mcpError = 'Token name is required.';
			return;
		}
		if (!Number.isFinite(newTokenExpiryDays) || newTokenExpiryDays < 1 || newTokenExpiryDays > 365) {
			mcpError = 'Expiry must be between 1 and 365 days.';
			return;
		}

		creatingMcpToken = true;
		try {
			const created = await createMcpToken({
				name: newTokenName.trim(),
				scopes: mcpScopesForAccess(newTokenAccess),
				expires_in_days: newTokenExpiryDays
			});
			createdToken = {
				name: created.name,
				token: created.token,
				scopes: created.scopes,
				expires_at: created.expires_at
			};
			newTokenName = '';
			newTokenAccess = 'read';
			newTokenExpiryDays = 30;
			mcpNotice = 'MCP token created. Copy it now; it will not be shown again.';
			await loadMcpTokens();
		} catch (e) {
			mcpError = e instanceof Error ? e.message : 'Failed to create MCP token';
		} finally {
			creatingMcpToken = false;
		}
	}

	async function copyCreatedToken() {
		if (typeof navigator === 'undefined' || !createdToken) return;
		try {
			await navigator.clipboard.writeText(createdToken.token);
			mcpNotice = 'MCP token copied to clipboard.';
		} catch {
			mcpError = 'Failed to copy token. Copy it manually.';
		}
	}

	async function handleRevokeMcpToken(token: McpTokenSummary) {
		if ($authState.status !== 'authenticated' || !$authState.user) return;
		if ($authState.offline) {
			mcpError = 'Offline mode: revoke MCP access tokens when online.';
			return;
		}
		if (!confirm(`Revoke MCP token "${token.name}"?`)) return;

		mcpError = null;
		mcpNotice = null;
		revokingTokenId = token.id;
		try {
			await revokeMcpToken(token.id);
			mcpNotice = `${token.name} revoked.`;
			await loadMcpTokens();
		} catch (e) {
			mcpError = e instanceof Error ? e.message : 'Failed to revoke MCP token';
		} finally {
			revokingTokenId = null;
		}
	}

	async function handleRotateMcpToken(token: McpTokenSummary) {
		if ($authState.status !== 'authenticated' || !$authState.user) return;
		if ($authState.offline) {
			mcpError = 'Offline mode: rotate MCP access tokens when online.';
			return;
		}
		if (!confirm(`Rotate MCP token "${token.name}"? The old token will stop working immediately.`)) {
			return;
		}

		mcpError = null;
		mcpNotice = null;
		createdToken = null;
		rotatingTokenId = token.id;
		try {
			const rotated = await rotateMcpToken(token.id);
			createdToken = {
				name: rotated.name,
				token: rotated.token,
				scopes: rotated.scopes,
				expires_at: rotated.expires_at
			};
			mcpNotice = `${token.name} rotated. Copy the new token now; the previous token no longer works.`;
			await loadMcpTokens();
		} catch (e) {
			mcpError = e instanceof Error ? e.message : 'Failed to rotate MCP token';
		} finally {
			rotatingTokenId = null;
		}
	}

	async function handleChangePassword() {
		accountError = null;
		accountNotice = null;
		if (!currentPassword || !newPassword) {
			accountError = 'Current and new password are required.';
			return;
		}
		if (newPassword !== confirmPassword) {
			accountError = 'New passwords do not match.';
			return;
		}

		accountLoading = true;
		try {
			await auth.changePassword(currentPassword, newPassword);
			currentPassword = '';
			newPassword = '';
			confirmPassword = '';
			accountNotice = 'Password updated.';
		} catch (e) {
			accountError = e instanceof Error ? e.message : 'Failed to change password';
		} finally {
			accountLoading = false;
		}
	}

	onMount(() => {
		void loadMcpTokens();
	});

	$: activeMcpTokens = mcpTokens.filter((token) => !token.revoked_at);
</script>

<div class="space-y-6">
	<header
		class="relative overflow-hidden rounded-2xl border border-surface-200/50 dark:border-surface-700/50 bg-gradient-to-br from-primary-500/10 via-transparent to-tertiary-500/10 p-5 sm:p-6"
	>
		<div
			class="pointer-events-none absolute -top-24 -right-24 size-72 rounded-full blur-3xl bg-primary-500/15"
		></div>
		<div
			class="pointer-events-none absolute -bottom-24 -left-24 size-72 rounded-full blur-3xl bg-secondary-500/15"
		></div>

		<div class="relative flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
			<div class="space-y-1">
				<h1 class="text-3xl sm:text-4xl font-black tracking-tight">Settings</h1>
				<p class="text-sm sm:text-base opacity-80 max-w-prose">
					Manage your account and the MCP tokens your AI tools use to connect to SwoleMate.
				</p>
			</div>
			<div class="flex sm:justify-end">
				<a href="/help" class="btn variant-soft">Help & troubleshooting →</a>
			</div>
		</div>
	</header>

	<div class="grid gap-6 md:grid-cols-12">
		<section class="md:col-span-7 space-y-4 min-w-0">
			<div class="card variant-glass-surface p-4 space-y-3">
				<div>
					<h2 class="text-lg font-semibold tracking-tight">AI access</h2>
					<p class="text-sm opacity-70">
						Create scoped MCP tokens for AI tools. Tokens are shown once, scoped, and can be
						rotated or revoked from here.
					</p>
				</div>

				<form
					class="space-y-3"
					on:submit|preventDefault={() => {
						void handleCreateMcpToken();
					}}
				>
					<label class="space-y-1 block">
						<span class="text-sm font-semibold">Token name</span>
						<input
							class="input w-full"
							placeholder="Claude Desktop"
							bind:value={newTokenName}
							disabled={creatingMcpToken}
						/>
					</label>

					<div class="grid gap-3 sm:grid-cols-2">
						<label class="space-y-1 block">
							<span class="text-sm font-semibold">Access level</span>
							<select class="select w-full" bind:value={newTokenAccess} disabled={creatingMcpToken}>
								<option value="read">Read only</option>
								<option value="write">Read and write</option>
							</select>
						</label>
						<label class="space-y-1 block">
							<span class="text-sm font-semibold">Expiry (days)</span>
							<input
								type="number"
								min="1"
								max="365"
								class="input w-full"
								bind:value={newTokenExpiryDays}
								disabled={creatingMcpToken}
							/>
						</label>
					</div>

					<div class="text-xs opacity-70">
						Use <code>Authorization: Bearer smcp_...</code> with your MCP client. Default to read-only unless the
						tool needs write access.
					</div>
					{#if newTokenAccess === 'write'}
						<div
							class="rounded-xl border border-warning-200/60 bg-warning-50/70 p-3 text-xs dark:border-warning-900/60 dark:bg-warning-950/20"
						>
							Write tokens can change workout data. A shorter expiry such as 7 days is recommended,
							but the default remains 30 days unless you change it.
						</div>
					{/if}

					{#if mcpError}
						<div class="text-sm text-error-500">{mcpError}</div>
					{/if}
					{#if mcpNotice}
						<div class="text-sm text-success-500">{mcpNotice}</div>
					{/if}

					<div class="flex flex-col sm:flex-row gap-2">
						<button
							type="submit"
							class="btn variant-filled-primary flex-1"
							disabled={creatingMcpToken || $authState.status !== 'authenticated'}
						>
							{creatingMcpToken ? 'Creating…' : 'Create MCP token'}
						</button>
						<button
							type="button"
							class="btn variant-soft flex-1"
							on:click={() => {
								void loadMcpTokens();
							}}
							disabled={mcpLoading || $authState.status !== 'authenticated'}
						>
							{mcpLoading ? 'Refreshing…' : 'Refresh tokens'}
						</button>
					</div>
				</form>

				{#if createdToken}
					<div
						class="rounded-xl border border-success-200/60 bg-success-50/70 p-3 dark:border-success-900/60 dark:bg-success-950/20 space-y-2"
					>
						<div class="flex items-start justify-between gap-3">
							<div>
								<div class="font-semibold">Copy this token now</div>
								<div class="text-sm opacity-80">It will not be shown again after you leave this page.</div>
							</div>
							<button type="button" class="btn variant-soft-primary" on:click={copyCreatedToken}>
								Copy token
							</button>
						</div>
						<code class="block overflow-x-auto rounded-lg bg-surface-950/90 p-3 text-xs text-surface-50">
							{createdToken.token}
						</code>
						<div class="flex flex-wrap gap-2">
							{#each createdToken.scopes as scope}
								<span class="badge variant-soft-primary">{scope}</span>
							{/each}
						</div>
						<div class="text-xs opacity-75">{createdToken.name} · expires {formatDateTime(createdToken.expires_at)}</div>
					</div>
				{/if}
			</div>

			<div class="card variant-glass-surface p-4 space-y-3">
				<div>
					<h2 class="text-lg font-semibold tracking-tight">Account</h2>
					<p class="text-sm opacity-70">
						Sessions stay signed in for a long time so offline mode keeps working. You can still log
						out manually.
					</p>
				</div>

				{#if $authState.status === 'authenticated' && $authState.user?.must_change_password}
					<div class="alert variant-filled-warning">
						Password change required. Update your password before using the app.
					</div>
				{/if}

				{#if $authState.user}
					<div class="flex flex-wrap items-center gap-2">
						<span class="badge variant-filled-primary">{$authState.user.username}</span>
						<span class="badge variant-soft">{$authState.user.role}</span>
						{#if $authState.offline}
							<span class="badge variant-soft-warning">Offline</span>
						{/if}
					</div>
				{:else}
					<div class="text-sm opacity-75">Not signed in.</div>
				{/if}

				<form
					class="space-y-3"
					on:submit|preventDefault={() => {
						void handleChangePassword();
					}}
				>
					<div class="grid gap-3 sm:grid-cols-2">
						<label class="space-y-1 block">
							<span class="text-sm font-semibold">Current password</span>
							<input
								type="password"
								class="input w-full"
								autocomplete="current-password"
								bind:value={currentPassword}
								disabled={accountLoading || !$authState.user}
							/>
						</label>
						<label class="space-y-1 block">
							<span class="text-sm font-semibold">New password</span>
							<input
								type="password"
								class="input w-full"
								autocomplete="new-password"
								bind:value={newPassword}
								disabled={accountLoading || !$authState.user}
							/>
						</label>
					</div>

					<label class="space-y-1 block">
						<span class="text-sm font-semibold">Confirm new password</span>
						<input
							type="password"
							class="input w-full"
							autocomplete="new-password"
							bind:value={confirmPassword}
							disabled={accountLoading || !$authState.user}
						/>
					</label>

					{#if accountError}
						<div class="text-sm text-error-500">{accountError}</div>
					{/if}
					{#if accountNotice}
						<div class="text-sm text-success-500">{accountNotice}</div>
					{/if}

					<div class="flex flex-col sm:flex-row gap-2">
						<button
							type="submit"
							class="btn variant-filled-primary flex-1"
							disabled={accountLoading || !$authState.user}
						>
							{accountLoading ? 'Updating…' : 'Change password'}
						</button>
						<button
							type="button"
							class="btn variant-soft-error flex-1"
							disabled={accountLoading || !$authState.user}
							on:click={() => auth.logout()}
						>
							Log out
						</button>
					</div>
				</form>
			</div>
		</section>

		<aside class="md:col-span-5 space-y-4 min-w-0">
			<div class="card variant-glass-surface p-4 space-y-3">
				<div class="flex items-start justify-between gap-3">
					<div>
						<h2 class="text-lg font-semibold tracking-tight">Active MCP tokens</h2>
						<p class="text-sm opacity-70">
							These are the tokens your AI tools can currently use to access the MCP endpoint.
						</p>
					</div>
					<span class="badge variant-soft">{activeMcpTokens.length}</span>
				</div>

				{#if $authState.status !== 'authenticated'}
					<div class="text-sm opacity-75">Sign in to manage AI access.</div>
				{:else if $authState.offline}
					<div class="text-sm text-warning-500">Offline: token management is unavailable.</div>
				{:else if mcpLoading && activeMcpTokens.length === 0}
					<div class="text-sm opacity-75">Loading tokens…</div>
				{:else if activeMcpTokens.length === 0}
					<div class="text-sm opacity-75">No MCP tokens yet.</div>
				{:else}
					<div class="space-y-3">
						{#each activeMcpTokens as token (token.id)}
							<div
								class="rounded-xl border border-surface-200/50 bg-surface-50/60 p-3 dark:border-surface-700/50 dark:bg-surface-950/30 space-y-2"
							>
								<div class="flex items-start justify-between gap-3">
									<div>
										<div class="font-semibold">{token.name}</div>
										<div class="text-xs opacity-70">{describeTokenAccess(token.scopes)}</div>
									</div>
									<div class="flex flex-wrap gap-2 justify-end">
										<button
											type="button"
											class="btn variant-soft"
											disabled={rotatingTokenId === token.id || revokingTokenId === token.id}
											on:click={() => {
												void handleRotateMcpToken(token);
											}}
										>
											{rotatingTokenId === token.id ? 'Rotating…' : 'Rotate'}
										</button>
										<button
											type="button"
											class="btn variant-soft-error"
											disabled={revokingTokenId === token.id || rotatingTokenId === token.id}
											on:click={() => {
												void handleRevokeMcpToken(token);
											}}
										>
											{revokingTokenId === token.id ? 'Revoking…' : 'Revoke'}
										</button>
									</div>
								</div>
								<div class="flex flex-wrap gap-2">
									{#each token.scopes as scope}
										<span class="badge variant-soft-primary">{scope}</span>
									{/each}
								</div>
								<div class="grid gap-2 text-xs opacity-80 sm:grid-cols-2">
									<div class="rounded-lg bg-surface-50/70 p-2 dark:bg-surface-950/30">
										<div class="font-semibold opacity-90">Last used</div>
										<div>{formatDateTime(token.last_used_at)}</div>
									</div>
									<div class="rounded-lg bg-surface-50/70 p-2 dark:bg-surface-950/30">
										<div class="font-semibold opacity-90">Expires</div>
										<div>{formatDateTime(token.expires_at)}</div>
									</div>
									<div class="rounded-lg bg-surface-50/70 p-2 dark:bg-surface-950/30 sm:col-span-2">
										<div class="font-semibold opacity-90">Created</div>
										<div>{formatDateTime(token.created_at)}</div>
									</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</aside>
	</div>
</div>
