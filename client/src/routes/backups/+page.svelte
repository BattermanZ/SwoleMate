<script lang="ts">
	import { onMount } from 'svelte';
	import { getBackups, createBackup, restoreBackup, deleteBackup } from '$lib/api';
	import type { BackupInfo } from '$lib/api';
	import { logger } from '$lib/logger';

	export let data: { backups: BackupInfo[] };
	let backups = data.backups;
	let loading = false;
	let error: string | null = null;

	async function loadBackups() {
		try {
			loading = true;
			error = null;
			backups = await getBackups();
			logger.info('backups', 'Loaded backups', { count: backups.length });
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load backups';
			logger.error('backups', 'Failed to load backups', { error });
		} finally {
			loading = false;
		}
	}

	async function handleCreateBackup() {
		try {
			loading = true;
			error = null;
			await createBackup();
			logger.info('backups', 'Created manual backup');
			await loadBackups();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create backup';
			logger.error('backups', 'Failed to create backup', { error });
		} finally {
			loading = false;
		}
	}

	async function handleRestore(filename: string) {
		if (
			!confirm(
				'Are you sure you want to restore this backup? This will replace your current database.'
			)
		) {
			return;
		}

		try {
			loading = true;
			error = null;
			await restoreBackup(filename);
			logger.info('backups', 'Restored backup successfully');
			window.location.reload();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to restore backup';
			logger.error('backups', 'Failed to restore backup', { error });
		} finally {
			loading = false;
		}
	}

	async function handleDelete(filename: string) {
		if (!confirm('Are you sure you want to delete this backup? This cannot be undone.')) {
			return;
		}

		try {
			loading = true;
			error = null;
			await deleteBackup(filename);
			backups = backups.filter((b) => b.filename !== filename);
			logger.info('backups', 'Deleted backup successfully');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete backup';
			logger.error('backups', 'Failed to delete backup', { error });
		} finally {
			loading = false;
		}
	}

	function formatDate(date: Date): string {
		const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
		const months = [
			'January',
			'February',
			'March',
			'April',
			'May',
			'June',
			'July',
			'August',
			'September',
			'October',
			'November',
			'December'
		];

		const getOrdinal = (n: number) => {
			const s = ['th', 'st', 'nd', 'rd'];
			const v = n % 100;
			return n + (s[(v - 20) % 10] || s[v] || s[0]);
		};

		return `${days[date.getDay()]}, ${getOrdinal(date.getDate())} of ${months[date.getMonth()]} at ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
	}

	onMount(loadBackups);
</script>

<div class="container mx-auto p-4 space-y-6">
	<header class="text-center space-y-4">
		<h2 class="h2">Database Backups</h2>
		<div class="flex justify-center">
			<button class="btn variant-filled-primary" on:click={handleCreateBackup} disabled={loading}>
				<span class="text-xl mr-2">💾</span>
				Create Manual Backup
			</button>
		</div>
	</header>

	{#if error}
		<div class="alert variant-filled-error">
			{error}
		</div>
	{/if}

	<div class="grid gap-4">
		{#if loading}
			<div class="card variant-ghost p-4 text-center">
				<span class="loading">Loading backups...</span>
			</div>
		{:else if backups.length === 0}
			<div class="card variant-ghost p-4 text-center">
				<p>No backups available yet.</p>
			</div>
		{:else}
			<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
				{#each backups as backup}
					<div class="card variant-glass-surface p-4 hover:variant-glass-surface transition-colors">
						<div class="flex flex-col h-full">
							<div class="flex-1">
								<div class="flex items-center justify-between mb-2">
									<span class="text-base font-medium">
										{formatDate(new Date(backup.created_at))}
									</span>
								</div>
								<div class="mt-2">
									<span
										class="badge {backup.backup_type === 'Auto'
											? 'variant-filled-secondary'
											: 'variant-filled-primary'}"
									>
										{backup.backup_type}
									</span>
									<span class="text-sm ml-2 opacity-60">
										{backup.filename.replace('swolemate_backup_', '').replace('.tar.gz', '')}
									</span>
								</div>
							</div>
							<div class="flex gap-2 mt-4">
								<button
									class="btn variant-filled-warning flex-1"
									on:click={() => handleRestore(backup.filename)}
									disabled={loading}
								>
									<span class="text-xl mr-2">🔄</span>
									Restore
								</button>
								<button
									class="btn variant-filled-error flex-1"
									on:click={() => handleDelete(backup.filename)}
									disabled={loading}
								>
									<span class="text-xl mr-2">🗑️</span>
									Delete
								</button>
							</div>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
