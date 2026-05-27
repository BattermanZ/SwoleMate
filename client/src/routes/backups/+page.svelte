<script lang="ts">
	import { createBackup, deleteBackup, getBackups, restoreBackup, type BackupInfo } from '$lib/api';
	import { auth } from '$lib/auth';
	import { Btn, Card, Badge, PageHero, Notice } from '$lib/components/ui';
	import BackupsDesktop from '$lib/components/backups/BackupsDesktop.svelte';
	import { openConfirm } from '$lib/stores/confirm';
	import { isDesktop, isDesktopView } from '$lib/stores/viewport';

	interface Props {
		data: { backups: BackupInfo[] };
	}
	let { data }: Props = $props();

	let desktop = $derived(isDesktopView($isDesktop));

	let backups = $derived(data.backups);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let notice = $state<string | null>(null);

	const authState = auth.state;

	async function refresh() {
		loading = true;
		error = null;
		try {
			backups = await getBackups();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load backups';
		} finally {
			loading = false;
		}
	}

	async function createNow() {
		loading = true;
		error = notice = null;
		try {
			const created = await createBackup();
			notice = `Backup created: ${created.filename}`;
			await refresh();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create backup';
		} finally {
			loading = false;
		}
	}

	async function handleRestore(b: BackupInfo) {
		if (
			!(await openConfirm({
				title: 'Restore this backup?',
				message: `Your current data will be REPLACED with ${b.filename}.`,
				confirmLabel: 'Restore',
				danger: true
			}))
		)
			return;
		loading = true;
		error = notice = null;
		try {
			await restoreBackup(b.filename);
			notice = `Restored from ${b.filename}.`;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to restore';
		} finally {
			loading = false;
		}
	}

	async function handleDelete(b: BackupInfo) {
		if (
			!(await openConfirm({
				title: 'Delete backup?',
				message: `${b.filename} cannot be recovered.`,
				confirmLabel: 'Delete',
				danger: true
			}))
		)
			return;
		loading = true;
		error = notice = null;
		try {
			await deleteBackup(b.filename);
			notice = `Deleted ${b.filename}.`;
			await refresh();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete backup';
		} finally {
			loading = false;
		}
	}

	function formatDateTime(value: string | null | undefined): string {
		if (!value) return '—';
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

	let autoBackups = $derived(backups.filter((b) => b.backup_type === 'Auto'));
	let manualBackups = $derived(backups.filter((b) => b.backup_type === 'Manual'));
</script>

{#snippet hero()}
	<PageHero kicker="► Data & backups">
		{#snippet title()}Snapshot, <em>restore.</em>{/snippet}
		{#snippet sub()}Manual + automatic backups of your training database.{/snippet}
	</PageHero>
{/snippet}

{#snippet actionsCard()}
	<Card>
		{#snippet title()}Actions{/snippet}
		{#snippet lede()}Trigger a manual snapshot or refresh the backup list.{/snippet}
		{#if error}<Notice tone="error">{error}</Notice>{/if}
		{#if notice}<Notice tone="success">{notice}</Notice>{/if}
		<div class="actions">
			<Btn variant="primary" onclick={createNow} disabled={loading || $authState.offline}>
				{loading ? 'Working…' : '+ New backup'}
			</Btn>
			<Btn variant="soft" onclick={refresh} disabled={loading}>
				{loading ? 'Loading…' : '↻ Refresh'}
			</Btn>
		</div>
	</Card>
{/snippet}

{#snippet manualCard()}
	<Card>
		{#snippet title()}Manual backups{/snippet}
		{#if manualBackups.length === 0}
			<div class="muted">No manual backups yet.</div>
		{:else}
			<div class="list">
				{#each manualBackups as b (b.filename)}
					<div class="b-row">
						<div class="info">
							<div class="t">{b.filename}</div>
							<div class="meta">
								<Badge tone="pr">Manual</Badge>
								<span>{formatDateTime(b.created_at)}</span>
								<span>·</span>
								<span>{b.backup_type}</span>
							</div>
						</div>
						<div class="actions">
							<Btn variant="soft" size="sm" onclick={() => handleRestore(b)} disabled={loading}>
								Restore
							</Btn>
							<Btn variant="soft" size="sm" onclick={() => handleDelete(b)} disabled={loading}>
								Delete
							</Btn>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</Card>
{/snippet}

{#snippet autoCard()}
	<Card>
		{#snippet title()}Automatic backups{/snippet}
		{#snippet lede()}Auto-snapshots run weekly. Older ones are pruned.{/snippet}
		{#if autoBackups.length === 0}
			<div class="muted">No automatic backups yet.</div>
		{:else}
			<div class="list">
				{#each autoBackups as b (b.filename)}
					<div class="b-row">
						<div class="info">
							<div class="t">{b.filename}</div>
							<div class="meta">
								<Badge tone="soft">Auto</Badge>
								<span>{formatDateTime(b.created_at)}</span>
								<span>·</span>
								<span>{b.backup_type}</span>
							</div>
						</div>
						<div class="actions">
							<Btn variant="soft" size="sm" onclick={() => handleRestore(b)} disabled={loading}>
								Restore
							</Btn>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</Card>
{/snippet}

{#if desktop}
	<BackupsDesktop {hero} {actionsCard} {manualCard} {autoCard} />
{:else}
	<div class="page">
		{@render hero()}
		{@render actionsCard()}
		{@render manualCard()}
		{@render autoCard()}
	</div>
{/if}

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.muted {
		font:
			500 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.b-row {
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 12px;
		padding: 12px;
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 10px;
		align-items: center;
	}
	.info {
		min-width: 0;
	}
	.t {
		font:
			700 13px/1.2 'JetBrains Mono',
			monospace;
		color: var(--ink);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.meta {
		margin-top: 6px;
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
		font:
			500 12px/1.3 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
		align-items: center;
	}
</style>
