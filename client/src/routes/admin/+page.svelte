<script lang="ts">
	import { onMount } from 'svelte';
	import { auth } from '$lib/auth';
	import {
		adminCreateUser,
		adminDeleteUser,
		adminDisableUser,
		adminListUsers,
		adminResetUserPassword,
		type AdminUserListItem
	} from '$lib/api';
	import { Btn, Card, Badge, PageHero } from '$lib/components/ui';

	const authState = auth.state;

	let users = $state<AdminUserListItem[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let notice = $state<string | null>(null);

	let createUsername = $state('');
	let createPassword = $state('');
	let createRole = $state<'user' | 'admin'>('user');

	let resetTarget = $state<AdminUserListItem | null>(null);
	let resetPassword = $state('');
	let resetConfirm = $state('');
	let resetError = $state<string | null>(null);

	let isAdmin = $derived($authState.user?.role === 'admin');
	let blocked = $derived(
		$authState.offline || $authState.status !== 'authenticated' || !isAdmin
	);

	async function loadUsers() {
		if (blocked) return;
		loading = true;
		error = null;
		try {
			users = await adminListUsers();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load users';
		} finally {
			loading = false;
		}
	}

	async function handleCreateUser() {
		if (blocked) return;
		error = notice = null;
		if (!createUsername.trim() || !createPassword) {
			error = 'Username and password are required.';
			return;
		}
		loading = true;
		try {
			await adminCreateUser({
				username: createUsername,
				password: createPassword,
				role: createRole
			});
			createUsername = createPassword = '';
			createRole = 'user';
			notice = 'User created.';
			await loadUsers();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create user';
		} finally {
			loading = false;
		}
	}

	async function handleDisable(u: AdminUserListItem) {
		if (blocked) return;
		if (!confirm(`Disable ${u.username}? They will not be able to sign in.`)) return;
		loading = true;
		error = notice = null;
		try {
			await adminDisableUser(u.id);
			notice = `${u.username} disabled.`;
			await loadUsers();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to disable user';
		} finally {
			loading = false;
		}
	}

	async function handleDelete(u: AdminUserListItem) {
		if (blocked) return;
		if (
			!confirm(
				`Delete ${u.username}? This removes ALL their data and cannot be undone.`
			)
		) {
			return;
		}
		loading = true;
		error = notice = null;
		try {
			await adminDeleteUser(u.id);
			notice = `${u.username} deleted.`;
			await loadUsers();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete user';
		} finally {
			loading = false;
		}
	}

	async function handleReset() {
		if (!resetTarget) return;
		resetError = null;
		if (!resetPassword) {
			resetError = 'New password is required.';
			return;
		}
		if (resetPassword !== resetConfirm) {
			resetError = 'Passwords do not match.';
			return;
		}
		loading = true;
		try {
			await adminResetUserPassword(resetTarget.id, resetPassword);
			notice = `Password reset for ${resetTarget.username}.`;
			resetTarget = null;
			resetPassword = resetConfirm = '';
			await loadUsers();
		} catch (e) {
			resetError = e instanceof Error ? e.message : 'Failed to reset password';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		void loadUsers();
	});
</script>

<div class="page">
	<PageHero kicker="► Admin">
		{#snippet title()}Manage <em>users.</em>{/snippet}
		{#snippet sub()}Create, reset, disable, or delete accounts. Admins only.{/snippet}
	</PageHero>

	{#if !isAdmin}
		<Card>
			<div class="muted">You need admin privileges to view this page.</div>
		</Card>
	{:else if $authState.offline}
		<Card>
			<div class="muted">Offline mode — admin actions are disabled.</div>
		</Card>
	{:else}
		<Card>
			{#snippet title()}Create user{/snippet}
			<form
				class="form"
				onsubmit={(e) => {
					e.preventDefault();
					void handleCreateUser();
				}}
			>
				<div class="grid-2">
					<label>
						<span class="lbl">Username</span>
						<input bind:value={createUsername} disabled={loading} autocomplete="off" />
					</label>
					<label>
						<span class="lbl">Password</span>
						<input
							type="password"
							bind:value={createPassword}
							disabled={loading}
							autocomplete="new-password"
						/>
					</label>
				</div>
				<label>
					<span class="lbl">Role</span>
					<select bind:value={createRole} disabled={loading}>
						<option value="user">User</option>
						<option value="admin">Admin</option>
					</select>
				</label>
				{#if error}<div class="err">{error}</div>{/if}
				{#if notice}<div class="ok">{notice}</div>{/if}
				<Btn variant="primary" type="submit" disabled={loading}>
					{loading ? 'Creating…' : 'Create user'}
				</Btn>
			</form>
		</Card>

		<Card>
			{#snippet title()}Users{/snippet}
			{#snippet actions()}
				<Btn variant="soft" size="sm" onclick={loadUsers} disabled={loading}>
					{loading ? 'Loading…' : 'Refresh'}
				</Btn>
			{/snippet}

			{#if users.length === 0}
				<div class="muted">No users yet.</div>
			{:else}
				<div class="users">
					{#each users as u (u.id)}
						<div class="user">
							<div class="head">
								<div>
									<div class="t">{u.username}</div>
									<div class="meta">
										<Badge tone={u.role === 'admin' ? 'pr' : 'soft'}>{u.role}</Badge>
										{#if u.disabled_at}<Badge tone="warn">Disabled</Badge>{/if}
									</div>
								</div>
								<div class="actions">
									<Btn
										variant="soft"
										size="sm"
										disabled={loading}
										onclick={() => (resetTarget = u)}
									>
										Reset pw
									</Btn>
									<Btn
										variant="soft"
										size="sm"
										disabled={loading || !!u.disabled_at}
										onclick={() => handleDisable(u)}
									>
										Disable
									</Btn>
									<Btn variant="soft" size="sm" disabled={loading} onclick={() => handleDelete(u)}>
										Delete
									</Btn>
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</Card>

		{#if resetTarget}
			{@const target = resetTarget}
			<Card>
				{#snippet title()}Reset password <em>— {target.username}</em>{/snippet}
				<form
					class="form"
					onsubmit={(e) => {
						e.preventDefault();
						void handleReset();
					}}
				>
					<label>
						<span class="lbl">New password</span>
						<input type="password" bind:value={resetPassword} autocomplete="new-password" />
					</label>
					<label>
						<span class="lbl">Confirm</span>
						<input type="password" bind:value={resetConfirm} autocomplete="new-password" />
					</label>
					{#if resetError}<div class="err">{resetError}</div>{/if}
					<div class="actions">
						<Btn variant="primary" type="submit" disabled={loading}>
							{loading ? 'Resetting…' : 'Reset password'}
						</Btn>
						<Btn
							variant="soft"
							onclick={() => {
								resetTarget = null;
								resetPassword = resetConfirm = '';
								resetError = null;
							}}
						>Cancel</Btn>
					</div>
				</form>
			</Card>
		{/if}
	{/if}
</div>

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
		font: 700 10px/1 'Onest', system-ui, sans-serif;
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
		font: 500 14px/1.2 'Onest', system-ui, sans-serif;
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
		flex-wrap: wrap;
		gap: 8px;
	}
	.err {
		font: 600 13px/1.4 'Onest', system-ui, sans-serif;
		color: var(--clay-text);
	}
	.ok {
		font: 600 13px/1.4 'Onest', system-ui, sans-serif;
		color: var(--sage);
	}
	.muted {
		font: 500 13px/1.4 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
	}

	.users {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.user {
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 12px;
		padding: 12px;
	}
	.user .head {
		display: flex;
		justify-content: space-between;
		align-items: start;
		gap: 10px;
		flex-wrap: wrap;
	}
	.user .t {
		font: 800 14px/1 'Onest', system-ui, sans-serif;
	}
	.user .meta {
		margin-top: 6px;
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}
</style>
