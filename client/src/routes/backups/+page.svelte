<script lang="ts">
	import {
		getBackups,
		createBackup,
		restoreBackup,
		deleteBackup,
		getWorkouts,
		getWorkout
	} from '$lib/api';
	import type { BackupInfo } from '$lib/api';
	import { browser } from '$app/environment';
	import { auth } from '$lib/auth';
	import { logger } from '$lib/logger';
	import { formatDateLongWithTime } from '$lib/utils/date';

	export let data: { backups: BackupInfo[] };
	let backups = data.backups;
	let loading = false;
	let error: string | null = null;
	let exporting = false;
	let exportError: string | null = null;

	const authState = auth.state;
	$: isAdmin = $authState.user?.role === 'admin';
	$: canUseAdminEndpoints = isAdmin && !$authState.offline;

	function formatBackupFilename(filename: string): string {
		// New format: swolemate_YYYY-MM-DD_HH-mm_<auto|manual>(-N).tar.gz
		const next =
			/^swolemate_(\d{4}-\d{2}-\d{2})_(\d{2})-(\d{2})_(auto|manual)(?:-(\d+))?\.tar\.gz$/i.exec(
				filename
			);
		if (next) {
			const [, date, hh, mm, , dup] = next;
			const suffix = dup ? ` #${Number(dup) + 1}` : '';
			return `${date} ${hh}:${mm}${suffix}`;
		}

		// Legacy format: swolemate_backup_YYYYMMDD_HHMMSS_<auto|manual>_<timestamp>.tar.gz
		const legacy =
			/^swolemate_backup_(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})_(auto|manual)_(\d+)\.tar\.gz$/i.exec(
				filename
			);
		if (legacy) {
			const [, yyyy, mo, dd, hh, mm, ss] = legacy;
			return `${yyyy}-${mo}-${dd} ${hh}:${mm}:${ss}`;
		}

		return filename.replace(/\.tar\.gz$/i, '');
	}

	const AUTO_BACKUP_KEEP_WEEKLY = 6;
	const AUTO_BACKUP_KEEP_MONTHS = 6;
	const AUTO_BACKUP_KEEP_MAX = 12;
	const AUTO_BACKUP_DAY_LABEL = 'Monday';
	const AUTO_BACKUP_TIME_LABEL = '01:00';

	function nextAutoBackupLabel(now: Date = new Date()): string {
		const next = new Date(now);
		const day = next.getDay(); // 0 Sun .. 1 Mon .. 6 Sat
		const daysUntilMonday = (8 - day) % 7;
		next.setDate(next.getDate() + daysUntilMonday);
		next.setHours(1, 0, 0, 0);
		if (next.getTime() <= now.getTime()) next.setDate(next.getDate() + 7);
		const date = next.toLocaleDateString('en-GB', { day: '2-digit', month: '2-digit' });
		return `${AUTO_BACKUP_DAY_LABEL} ${date} ${AUTO_BACKUP_TIME_LABEL}`;
	}

	async function loadBackups() {
		if (!canUseAdminEndpoints) return;
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
		if (!canUseAdminEndpoints) return;
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
		if (!canUseAdminEndpoints) return;
		if (
			!confirm(
				'Are you sure you want to restore this backup? This will replace your current database.'
			)
		) {
			return;
		}

		if ($authState.offline) {
			error = 'Offline mode: restore backups when online.';
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
		if (!canUseAdminEndpoints) return;
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

	async function exportAllData() {
		if (!browser) return;
		if (exporting) return;
		if (!canUseAdminEndpoints) return;

		try {
			exporting = true;
			exportError = null;

			const workouts = await getWorkouts();
			const detailed = await Promise.all(
				workouts
					.filter((w) => typeof w.id === 'number')
					.map(async (w) => {
						const data = await getWorkout(w.id!);
						return { ...data.workout, exercises: data.exercises };
					})
			);

			const payload = {
				exported_at: new Date().toISOString(),
				workouts: detailed
			};

			const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' });
			const url = window.URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `swolemate-export-${new Date().toISOString().slice(0, 10)}.json`;
			a.click();
			window.URL.revokeObjectURL(url);
		} catch (e) {
			const message = e instanceof Error ? e.message : 'Failed to export data';
			exportError = message;
			logger.error('backups', 'Failed to export data', { error: e });
		} finally {
			exporting = false;
		}
	}

	$: if (canUseAdminEndpoints) {
		void loadBackups();
	}
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
				<h1 class="text-3xl sm:text-4xl font-black tracking-tight">Backups</h1>
				<p class="text-sm sm:text-base opacity-80 max-w-prose">
					Protect your database and export data for portability.
				</p>
			</div>
			<div class="flex flex-col sm:items-end gap-2">
				<div class="flex flex-col sm:flex-row gap-2 w-full sm:w-auto">
					<button
						type="button"
						class="btn variant-filled-primary w-full sm:w-auto"
						on:click={handleCreateBackup}
						disabled={loading || !canUseAdminEndpoints}
					>
						Create backup
					</button>
					<button
						type="button"
						class="btn variant-soft w-full sm:w-auto"
						on:click={exportAllData}
						disabled={exporting || !canUseAdminEndpoints}
					>
						{exporting ? 'Exporting…' : 'Export JSON'}
					</button>
				</div>
				{#if $authState.offline}
					<div class="text-sm text-warning-500">Offline: backups are unavailable.</div>
				{:else if $authState.status === 'unauthenticated'}
					<div class="text-sm text-warning-500">Sign in to manage backups.</div>
				{:else if !isAdmin}
					<div class="text-sm text-warning-500">Admin only.</div>
				{/if}
				{#if error || exportError}
					<div class="text-sm text-error-500">{error ?? exportError}</div>
				{/if}
			</div>
		</div>
	</header>

	<div class="grid gap-6 md:grid-cols-12">
		<section class="md:col-span-8 space-y-4 min-w-0">
			{#if !canUseAdminEndpoints}
				<div class="card variant-ghost p-4 text-center opacity-80">
					Backups are available only to admin accounts.
				</div>
			{:else if loading}
				<div class="card variant-ghost p-4 text-center opacity-80">Loading backups…</div>
			{:else if backups.length === 0}
				<div class="card variant-ghost p-4 text-center opacity-80">No backups available yet.</div>
			{:else}
				<div class="grid gap-4 sm:grid-cols-2">
					{#each backups as backup}
						<div class="card variant-glass-surface p-4 min-w-0">
							<div class="flex items-start justify-between gap-3">
								<div class="min-w-0">
									<div class="text-sm font-semibold truncate">
										{formatDateLongWithTime(new Date(backup.created_at))}
									</div>
									<div class="mt-2 flex flex-wrap items-center gap-2">
										<span
											class="badge {backup.backup_type === 'Auto'
												? 'variant-filled-secondary'
												: 'variant-filled-primary'}"
										>
											{backup.backup_type}
										</span>
										<span class="text-xs opacity-70 truncate">
											{formatBackupFilename(backup.filename)}
										</span>
									</div>
								</div>
							</div>

							<div class="flex gap-2 mt-4">
								<button
									type="button"
									class="btn variant-filled-warning flex-1"
									on:click={() => handleRestore(backup.filename)}
									disabled={loading || !canUseAdminEndpoints}
								>
									Restore
								</button>
								<button
									type="button"
									class="btn variant-soft-error flex-1"
									on:click={() => handleDelete(backup.filename)}
									disabled={loading || !canUseAdminEndpoints}
								>
									Delete
								</button>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</section>

		<aside class="md:col-span-4 space-y-4 min-w-0">
			<div class="card variant-glass-surface p-4 space-y-2">
				<h2 class="text-lg font-semibold tracking-tight">Auto backup policy</h2>
				<ul class="text-sm opacity-80 space-y-1 list-disc pl-5">
					<li>
						Runs every {AUTO_BACKUP_DAY_LABEL} at {AUTO_BACKUP_TIME_LABEL} (server local time).
					</li>
					<li>
						Keeps up to {AUTO_BACKUP_KEEP_MAX} auto backups: last {AUTO_BACKUP_KEEP_WEEKLY} weekly, plus
						1 per month for {AUTO_BACKUP_KEEP_MONTHS} months.
					</li>
					<li>Manual backups are kept until you delete them.</li>
				</ul>
				<div class="mt-2 text-xs opacity-70">Next auto backup: {nextAutoBackupLabel()}</div>
			</div>

			<div class="card variant-glass-surface p-4 space-y-2">
				<h2 class="text-lg font-semibold tracking-tight">Tips</h2>
				<ul class="text-sm opacity-80 space-y-1 list-disc pl-5">
					<li>Create a manual backup before updates or big changes.</li>
					<li>Restore replaces the current database (it’s destructive).</li>
					<li>Export JSON is a portable snapshot you can store anywhere.</li>
				</ul>
			</div>
			<a href="/settings" class="btn variant-soft w-full justify-center">Help →</a>
		</aside>
	</div>
</div>
