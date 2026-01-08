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

	const authState = auth.state;
	$: isAdmin = $authState.user?.role === 'admin';
	$: blocked = $authState.offline || $authState.status !== 'authenticated' || !isAdmin;

	let users: AdminUserListItem[] = [];
	let loading = false;
	let error: string | null = null;
	let notice: string | null = null;

	let createUsername = '';
	let createPassword = '';
	let createRole: 'user' | 'admin' = 'user';

	let resetTarget: AdminUserListItem | null = null;
	let resetPassword = '';
	let resetConfirm = '';
	let resetError: string | null = null;

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
		error = null;
		notice = null;
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
			createUsername = '';
			createPassword = '';
			createRole = 'user';
			notice = 'User created.';
			await loadUsers();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create user';
		} finally {
			loading = false;
		}
	}

	async function handleDisableUser(user: AdminUserListItem) {
		if (blocked) return;
		if (!confirm(`Disable ${user.username}? This will revoke all sessions for that user.`)) return;
		error = null;
		notice = null;
		loading = true;
		try {
			await adminDisableUser(user.id);
			notice = `${user.username} disabled.`;
			await loadUsers();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to disable user';
		} finally {
			loading = false;
		}
	}

	function openReset(user: AdminUserListItem) {
		resetTarget = user;
		resetPassword = '';
		resetConfirm = '';
		resetError = null;
	}

	async function submitReset() {
		if (blocked || !resetTarget) return;
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
		} catch (e) {
			resetError = e instanceof Error ? e.message : 'Failed to reset password';
		} finally {
			loading = false;
		}
	}

	async function handleDeleteUser(user: AdminUserListItem) {
		if (blocked) return;
		const confirmation = prompt(
			`This permanently deletes ${user.username} and all their workouts.\n\nType the username to confirm:`
		);
		if (confirmation !== user.username) return;

		error = null;
		notice = null;
		loading = true;
		try {
			await adminDeleteUser(user.id);
			notice = `${user.username} deleted.`;
			await loadUsers();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete user';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		void loadUsers();
	});
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
				<h1 class="text-3xl sm:text-4xl font-black tracking-tight">Admin</h1>
				<p class="text-sm sm:text-base opacity-80 max-w-prose">
					Manage users and access control. Backups stay on the Backups page.
				</p>
			</div>
			<div class="flex flex-col gap-2 sm:items-end">
				<button
					type="button"
					class="btn variant-soft"
					disabled={blocked || loading}
					on:click={loadUsers}
				>
					Refresh
				</button>
				{#if $authState.offline}
					<div class="text-sm text-warning-500">Offline: admin features are unavailable.</div>
				{/if}
			</div>
		</div>
	</header>

	{#if blocked}
		<div class="card variant-ghost p-4 text-center opacity-80">Admin access required.</div>
	{:else}
		{#if error}
			<div class="card variant-ghost p-4 text-error-500">{error}</div>
		{/if}
		{#if notice}
			<div class="card variant-ghost p-4 text-success-500">{notice}</div>
		{/if}

		<div class="grid gap-6 md:grid-cols-12">
			<section class="md:col-span-7 space-y-4 min-w-0">
				<div class="card variant-glass-surface p-4 space-y-3">
					<div>
						<h2 class="text-lg font-semibold tracking-tight">Create user</h2>
						<p class="text-sm opacity-70">One user per device is the intended model.</p>
					</div>

					<form
						class="space-y-3"
						on:submit|preventDefault={() => {
							void handleCreateUser();
						}}
					>
						<div class="grid gap-3 sm:grid-cols-2">
							<label class="space-y-1 block">
								<span class="text-sm font-semibold">Username</span>
								<input class="input w-full" bind:value={createUsername} disabled={loading} />
							</label>
							<label class="space-y-1 block">
								<span class="text-sm font-semibold">Password</span>
								<input
									type="password"
									class="input w-full"
									bind:value={createPassword}
									disabled={loading}
								/>
							</label>
						</div>

						<label class="space-y-1 block">
							<span class="text-sm font-semibold">Role</span>
							<select class="select w-full" bind:value={createRole} disabled={loading}>
								<option value="user">User</option>
								<option value="admin">Admin</option>
							</select>
						</label>

						<button type="submit" class="btn variant-filled-primary w-full" disabled={loading}>
							{loading ? 'Working…' : 'Create user'}
						</button>
					</form>
				</div>
			</section>

			<aside class="md:col-span-5 space-y-4 min-w-0">
				<div class="card variant-glass-surface p-4 space-y-3">
					<div>
						<h2 class="text-lg font-semibold tracking-tight">Users</h2>
						<p class="text-sm opacity-70">Reset passwords, disable accounts, or delete users.</p>
					</div>

					{#if loading && users.length === 0}
						<div class="text-sm opacity-70">Loading users…</div>
					{:else if users.length === 0}
						<div class="text-sm opacity-70">No users found.</div>
					{:else}
						<div class="space-y-2">
							{#each users as u}
								<div
									class="rounded-xl border border-surface-200/50 bg-surface-50/60 p-3 dark:border-surface-700/50 dark:bg-surface-950/30"
								>
									<div class="flex items-start justify-between gap-2">
										<div class="min-w-0">
											<div class="font-semibold truncate">{u.username}</div>
											<div class="text-xs opacity-70">
												{u.role}{u.disabled_at ? ' • disabled' : ''}
											</div>
										</div>
										<div class="flex gap-2">
											<button
												type="button"
												class="btn btn-sm variant-soft"
												disabled={loading}
												on:click={() => openReset(u)}
											>
												Reset password
											</button>
										</div>
									</div>

									<div class="mt-3 flex gap-2">
										<button
											type="button"
											class="btn btn-sm variant-soft-warning flex-1"
											disabled={loading || !!u.disabled_at}
											on:click={() => handleDisableUser(u)}
										>
											Disable
										</button>
										<button
											type="button"
											class="btn btn-sm variant-soft-error flex-1"
											disabled={loading}
											on:click={() => handleDeleteUser(u)}
										>
											Delete
										</button>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</aside>
		</div>
	{/if}

	{#if resetTarget}
		<div
			class="fixed inset-0 z-50 flex items-center justify-center p-4"
			role="dialog"
			aria-modal="true"
		>
			<button
				class="absolute inset-0 bg-black/50"
				aria-label="Close"
				on:click={() => (resetTarget = null)}
			></button>
			<div class="relative w-full max-w-md card variant-glass-surface p-4 space-y-3">
				<div class="space-y-1">
					<h3 class="text-lg font-semibold">Reset password</h3>
					<div class="text-sm opacity-70">{resetTarget.username}</div>
				</div>

				<label class="space-y-1 block">
					<span class="text-sm font-semibold">New password</span>
					<input
						type="password"
						class="input w-full"
						bind:value={resetPassword}
						disabled={loading}
					/>
				</label>
				<label class="space-y-1 block">
					<span class="text-sm font-semibold">Confirm password</span>
					<input
						type="password"
						class="input w-full"
						bind:value={resetConfirm}
						disabled={loading}
					/>
				</label>
				{#if resetError}
					<div class="text-sm text-error-500">{resetError}</div>
				{/if}

				<div class="flex gap-2">
					<button
						type="button"
						class="btn variant-soft flex-1"
						on:click={() => (resetTarget = null)}
					>
						Cancel
					</button>
					<button
						type="button"
						class="btn variant-filled-primary flex-1"
						disabled={loading}
						on:click={() => {
							void submitReset();
						}}
					>
						{loading ? 'Working…' : 'Reset'}
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>
