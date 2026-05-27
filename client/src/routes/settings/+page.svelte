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
	import { readDemoModePreference, writeDemoModePreference } from '$lib/preferences/demoMode';
	import { Btn, Card, Chk, Badge, PageHero, Notice } from '$lib/components/ui';
	import SettingsDesktop from '$lib/components/settings/SettingsDesktop.svelte';
	import { isDesktop, isDesktopView } from '$lib/stores/viewport';
	import { openConfirm } from '$lib/stores/confirm';

	const authState = auth.state;

	let desktop = $derived(isDesktopView($isDesktop));

	// Account
	let currentPassword = $state('');
	let newPassword = $state('');
	let confirmPassword = $state('');
	let accountLoading = $state(false);
	let accountError = $state<string | null>(null);
	let accountNotice = $state<string | null>(null);

	// MCP tokens
	let mcpTokens = $state<McpTokenSummary[]>([]);
	let mcpLoading = $state(false);
	let mcpError = $state<string | null>(null);
	let mcpNotice = $state<string | null>(null);
	let creatingMcpToken = $state(false);
	let revokingTokenId = $state<number | null>(null);
	let rotatingTokenId = $state<number | null>(null);
	let newTokenName = $state('');
	let newTokenAccess = $state<'read' | 'write'>('read');
	let newTokenExpiryDays = $state(30);
	let createdToken = $state<{
		name: string;
		token: string;
		scopes: string[];
		expires_at: string | null;
	} | null>(null);

	let demoModeEnabled = $state(false);
	let mcpTokensLoaded = $state(false);

	let activeMcpTokens = $derived(mcpTokens.filter((t) => !t.revoked_at));

	let mcpUrl = $derived(typeof window !== 'undefined' ? `${window.location.origin}/mcp` : '/mcp');

	function mcpScopesForAccess(access: 'read' | 'write'): string[] {
		if (access === 'write') return ['workouts.read', 'progress.read', 'workouts.write'];
		return ['workouts.read', 'progress.read'];
	}

	function formatDateTime(value: string | null): string {
		if (!value) return 'Never';
		const d = new Date(value);
		if (!Number.isFinite(d.getTime())) return value;
		return d.toLocaleString([], {
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
			mcpTokensLoaded = true;
		} catch (e) {
			mcpError = e instanceof Error ? e.message : 'Failed to load MCP tokens';
		} finally {
			mcpLoading = false;
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
			accountError = 'New password and confirmation do not match.';
			return;
		}
		accountLoading = true;
		try {
			await auth.changePassword(currentPassword, newPassword);
			accountNotice = 'Password updated.';
			currentPassword = newPassword = confirmPassword = '';
		} catch (e) {
			accountError = e instanceof Error ? e.message : 'Failed to update password';
		} finally {
			accountLoading = false;
		}
	}

	async function handleCreateMcpToken() {
		mcpError = mcpNotice = null;
		if (!newTokenName.trim()) {
			mcpError = 'Token name is required.';
			return;
		}
		creatingMcpToken = true;
		try {
			const expires_in_days =
				newTokenExpiryDays && newTokenExpiryDays > 0 ? newTokenExpiryDays : undefined;
			const result = await createMcpToken({
				name: newTokenName.trim(),
				scopes: mcpScopesForAccess(newTokenAccess),
				expires_in_days
			});
			createdToken = {
				name: result.name,
				token: result.token,
				scopes: result.scopes,
				expires_at: result.expires_at
			};
			newTokenName = '';
			mcpNotice = 'Token created. Copy it now — it will not be shown again.';
			await loadMcpTokens();
		} catch (e) {
			mcpError = e instanceof Error ? e.message : 'Failed to create token';
		} finally {
			creatingMcpToken = false;
		}
	}

	async function handleRotateMcpToken(token: McpTokenSummary) {
		rotatingTokenId = token.id;
		mcpError = mcpNotice = null;
		try {
			const next = await rotateMcpToken(token.id);
			createdToken = {
				name: next.name,
				token: next.token,
				scopes: next.scopes,
				expires_at: next.expires_at
			};
			mcpNotice = 'Token rotated. Copy the new value now.';
			await loadMcpTokens();
		} catch (e) {
			mcpError = e instanceof Error ? e.message : 'Failed to rotate token';
		} finally {
			rotatingTokenId = null;
		}
	}

	async function handleRevokeMcpToken(token: McpTokenSummary) {
		if (
			!(await openConfirm({
				title: `Revoke ${token.name}?`,
				message: 'Clients using it will lose access immediately.',
				confirmLabel: 'Revoke',
				danger: true
			}))
		)
			return;
		revokingTokenId = token.id;
		mcpError = mcpNotice = null;
		try {
			await revokeMcpToken(token.id);
			mcpNotice = 'Token revoked.';
			await loadMcpTokens();
		} catch (e) {
			mcpError = e instanceof Error ? e.message : 'Failed to revoke token';
		} finally {
			revokingTokenId = null;
		}
	}

	async function copyCreatedToken() {
		if (!createdToken) return;
		try {
			await navigator.clipboard?.writeText(createdToken.token);
			mcpNotice = 'Token copied to clipboard.';
		} catch {
			mcpNotice = 'Could not copy — select the value manually.';
		}
	}

	async function copyMcpUrl() {
		try {
			await navigator.clipboard?.writeText(mcpUrl);
			mcpNotice = 'Endpoint URL copied to clipboard.';
		} catch {
			mcpNotice = 'Could not copy — select the value manually.';
		}
	}

	function handleDemoModeToggle(enabled: boolean) {
		demoModeEnabled = enabled;
		writeDemoModePreference(enabled);
	}

	onMount(async () => {
		demoModeEnabled = readDemoModePreference();
	});

	$effect(() => {
		if (
			$authState.status === 'authenticated' &&
			!$authState.offline &&
			!mcpTokensLoaded &&
			!mcpLoading
		) {
			void loadMcpTokens();
		}
	});
</script>

{#snippet hero()}
	<PageHero kicker="► Settings">
		{#snippet title()}Your account, <em>tuned.</em>{/snippet}
		{#snippet sub()}Password, demo mode, and the MCP tokens AI tools use to reach SwoleMate.{/snippet}
	</PageHero>
{/snippet}

{#snippet accountCard()}
	<Card>
		{#snippet title()}Account{/snippet}
		{#snippet lede()}Sessions stay signed in for a long time so offline mode keeps working.{/snippet}

		{#if $authState.status === 'authenticated' && $authState.user?.must_change_password}
			<div class="warn">Password change required. Update your password before using the app.</div>
		{/if}

		{#if $authState.user}
			<div class="badges">
				<Badge tone="pr">{$authState.user.username}</Badge>
				<Badge tone="soft">{$authState.user.role}</Badge>
				{#if $authState.offline}<Badge tone="warn">Offline</Badge>{/if}
			</div>
		{:else}
			<div class="muted">Not signed in.</div>
		{/if}

		<form
			class="form"
			onsubmit={(e) => {
				e.preventDefault();
				void handleChangePassword();
			}}
		>
			<div class="grid-2">
				<label>
					<span class="lbl">Current password</span>
					<input
						type="password"
						bind:value={currentPassword}
						autocomplete="current-password"
						disabled={accountLoading || !$authState.user}
					/>
				</label>
				<label>
					<span class="lbl">New password</span>
					<input
						type="password"
						bind:value={newPassword}
						autocomplete="new-password"
						disabled={accountLoading || !$authState.user}
					/>
				</label>
			</div>
			<label>
				<span class="lbl">Confirm new password</span>
				<input
					type="password"
					bind:value={confirmPassword}
					autocomplete="new-password"
					disabled={accountLoading || !$authState.user}
				/>
			</label>

			{#if accountError}<Notice tone="error">{accountError}</Notice>{/if}
			{#if accountNotice}<Notice tone="success">{accountNotice}</Notice>{/if}

			<div class="actions">
				<Btn variant="primary" type="submit" disabled={accountLoading || !$authState.user}>
					{accountLoading ? 'Updating…' : 'Change password'}
				</Btn>
				<Btn variant="soft" onclick={() => auth.logout()} disabled={!$authState.user}>Log out</Btn>
			</div>
		</form>
	</Card>
{/snippet}

{#snippet toolsCard()}
	<Card>
		{#snippet title()}Workout tools{/snippet}
		{#snippet lede()}Hide demo session tools unless you want quick access for testing.{/snippet}

		<div class="toggle-row">
			<div>
				<div class="t">Show demo session tools</div>
				<p>Add the demo session action back to the Today page header.</p>
			</div>
			<Chk
				label={demoModeEnabled ? 'On' : 'Off'}
				checked={demoModeEnabled}
				onchange={handleDemoModeToggle}
			/>
		</div>
	</Card>
{/snippet}

{#snippet mcpCard()}
	<Card>
		{#snippet title()}AI access · MCP tokens{/snippet}
		{#snippet lede()}Tokens AI tools use to reach the MCP endpoint. Each token is shown once.{/snippet}

		<form
			class="form"
			onsubmit={(e) => {
				e.preventDefault();
				void handleCreateMcpToken();
			}}
		>
			<label>
				<span class="lbl">Name</span>
				<input
					bind:value={newTokenName}
					placeholder="Claude Desktop"
					disabled={creatingMcpToken || $authState.status !== 'authenticated'}
				/>
			</label>

			<div class="grid-2">
				<label>
					<span class="lbl">Access</span>
					<select bind:value={newTokenAccess} disabled={creatingMcpToken}>
						<option value="read">Read only</option>
						<option value="write">Read + write</option>
					</select>
				</label>
				<label>
					<span class="lbl">Expires (days)</span>
					<input
						type="number"
						min="1"
						bind:value={newTokenExpiryDays}
						disabled={creatingMcpToken}
					/>
				</label>
			</div>

			{#if newTokenAccess === 'write'}
				<div class="warn">
					Write tokens can change workout data. A shorter expiry such as 7 days is recommended.
				</div>
			{/if}
			{#if mcpError}<Notice tone="error">{mcpError}</Notice>{/if}
			{#if mcpNotice}<Notice tone="success">{mcpNotice}</Notice>{/if}

			<div class="actions">
				<Btn
					variant="primary"
					type="submit"
					disabled={creatingMcpToken || $authState.status !== 'authenticated'}
				>
					{creatingMcpToken ? 'Creating…' : 'Create token'}
				</Btn>
				<Btn variant="soft" onclick={loadMcpTokens} disabled={mcpLoading}>
					{mcpLoading ? 'Refreshing…' : 'Refresh'}
				</Btn>
			</div>
		</form>

		{#if createdToken}
			<div class="created">
				<div class="row">
					<div>
						<div class="t">Copy this token now</div>
						<p>It will not be shown again after you leave this page.</p>
					</div>
					<Btn variant="ink" size="sm" onclick={copyCreatedToken}>Copy</Btn>
				</div>
				<code class="value">{createdToken.token}</code>
				<div class="meta">
					{createdToken.name} · expires {formatDateTime(createdToken.expires_at)}
				</div>
			</div>
		{/if}

		<div class="tokens-list">
			{#if $authState.status !== 'authenticated'}
				<div class="muted">Sign in to manage AI access.</div>
			{:else if $authState.offline}
				<div class="warn-text">Offline: token management is unavailable.</div>
			{:else if mcpLoading && activeMcpTokens.length === 0}
				<div class="muted">Loading tokens…</div>
			{:else if activeMcpTokens.length === 0}
				<div class="muted">No MCP tokens yet.</div>
			{:else}
				<h4>Active</h4>
				{#each activeMcpTokens as token (token.id)}
					<div class="token">
						<div class="head">
							<div>
								<div class="t">{token.name}</div>
								<div class="sub">{describeTokenAccess(token.scopes)}</div>
							</div>
							<div class="actions">
								<Btn
									variant="soft"
									size="sm"
									disabled={rotatingTokenId === token.id || revokingTokenId === token.id}
									onclick={() => handleRotateMcpToken(token)}
								>
									{rotatingTokenId === token.id ? 'Rotating…' : 'Rotate'}
								</Btn>
								<Btn
									variant="soft"
									size="sm"
									disabled={revokingTokenId === token.id || rotatingTokenId === token.id}
									onclick={() => handleRevokeMcpToken(token)}
								>
									{revokingTokenId === token.id ? 'Revoking…' : 'Revoke'}
								</Btn>
							</div>
						</div>
						<div class="meta2">
							<span>Last used <b>{formatDateTime(token.last_used_at)}</b></span>
							<span>Expires <b>{formatDateTime(token.expires_at)}</b></span>
						</div>
					</div>
				{/each}
			{/if}
		</div>

		<div class="connect">
			<h4>Connecting an AI client</h4>
			<p>
				In your client (Claude Desktop or any MCP-capable tool), add a custom/remote MCP server
				pointing at the endpoint below and supply the token as a bearer credential.
			</p>
			<ol class="steps">
				<li>
					<span class="step-lbl">Endpoint URL</span>
					<div class="row">
						<code class="value">{mcpUrl}</code>
						<Btn variant="ink" size="sm" onclick={copyMcpUrl}>Copy</Btn>
					</div>
				</li>
				<li>
					<span class="step-lbl">Authorization header</span>
					<code class="value">Authorization: Bearer smcp_…</code>
					<p class="hint">Use a token minted above as the <b>smcp_…</b> value.</p>
				</li>
				<li>
					<span class="step-lbl">Access level</span>
					<p class="hint">
						Read or read + write is set by the token's scope chosen when you create it.
					</p>
				</li>
			</ol>
		</div>
	</Card>
{/snippet}

{#if desktop}
	<SettingsDesktop {hero} {accountCard} {toolsCard} {mcpCard} />
{:else}
	<div class="page">
		{@render hero()}
		{@render accountCard()}
		{@render toolsCard()}
		{@render mcpCard()}
	</div>
{/if}

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.form {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-top: 8px;
	}
	.grid-2 {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}
	@media (max-width: 480px) {
		.grid-2 {
			grid-template-columns: 1fr;
		}
	}
	label {
		display: block;
	}
	.lbl {
		display: block;
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
		margin-bottom: 6px;
	}
	input,
	select {
		width: 100%;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 10px;
		padding: 11px 12px;
		font:
			500 14px/1.2 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink);
		outline: 0;
	}
	input:focus,
	select:focus {
		border-color: var(--clay);
		box-shadow: 0 0 0 3px rgba(255, 94, 31, 0.16);
	}
	.actions {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}
	.badges {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
		margin: 6px 0;
	}
	.warn {
		padding: 10px 12px;
		border-radius: 10px;
		background: color-mix(in oklab, var(--warn) 14%, var(--card));
		border: 1px solid color-mix(in oklab, var(--warn) 30%, var(--line));
		color: var(--warn);
		font:
			600 12px/1.4 'Onest',
			system-ui,
			sans-serif;
	}
	.warn-text {
		font:
			600 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--warn);
	}
	.muted {
		font:
			500 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}

	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 12px;
		padding: 12px 14px;
	}
	.toggle-row .t {
		font:
			800 14px/1 'Onest',
			system-ui,
			sans-serif;
	}
	.toggle-row p {
		margin: 4px 0 0;
		font:
			500 12px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}

	.created {
		margin-top: 12px;
		padding: 12px;
		border-radius: 12px;
		background: color-mix(in oklab, var(--sage) 10%, var(--card));
		border: 1px solid color-mix(in oklab, var(--sage) 30%, var(--line));
	}
	.created .row {
		display: flex;
		align-items: start;
		justify-content: space-between;
		gap: 10px;
	}
	.created .t {
		font:
			800 13px/1 'Onest',
			system-ui,
			sans-serif;
	}
	.created p {
		margin: 4px 0 0;
		font:
			500 12px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.created .value {
		display: block;
		margin-top: 10px;
		padding: 10px 12px;
		background: var(--surface-deep);
		color: var(--on-deep);
		border-radius: 10px;
		font:
			500 12px/1.5 'JetBrains Mono',
			monospace;
		overflow-x: auto;
	}
	.created .meta {
		margin-top: 8px;
		font: italic 400 12px/1.3 'Instrument Serif';
		color: var(--ink-soft);
	}

	.tokens-list {
		margin-top: 14px;
	}
	.tokens-list h4 {
		margin: 0 0 8px;
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
	}
	.token {
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 12px;
		padding: 12px;
	}
	.token + .token {
		margin-top: 8px;
	}
	.token .head {
		display: flex;
		justify-content: space-between;
		gap: 10px;
		align-items: start;
	}
	.token .t {
		font:
			800 14px/1 'Onest',
			system-ui,
			sans-serif;
	}
	.token .sub {
		margin-top: 4px;
		font:
			600 11px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.meta2 {
		margin-top: 8px;
		display: flex;
		gap: 12px;
		font:
			500 11px/1.3 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
		flex-wrap: wrap;
	}
	.meta2 b {
		color: var(--ink);
		font-weight: 700;
	}

	.connect {
		margin-top: 14px;
		padding-top: 14px;
		border-top: 1px solid var(--line);
	}
	.connect h4 {
		margin: 0 0 6px;
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
	}
	.connect > p {
		margin: 0 0 12px;
		font:
			500 13px/1.5 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.steps {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.steps .step-lbl {
		display: block;
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
		margin-bottom: 6px;
	}
	.steps .row {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.steps .row .value {
		flex: 1;
		margin: 0;
	}
	.steps .value {
		display: block;
		padding: 10px 12px;
		background: var(--surface-deep);
		color: var(--on-deep);
		border-radius: 10px;
		font:
			500 12px/1.5 'JetBrains Mono',
			monospace;
		overflow-x: auto;
	}
	.steps .hint {
		margin: 6px 0 0;
		font:
			500 12px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.steps .hint b {
		color: var(--ink);
		font-weight: 700;
	}
</style>
