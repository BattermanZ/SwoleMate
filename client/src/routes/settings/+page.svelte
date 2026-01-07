<script lang="ts">
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import { writable } from 'svelte/store';
	import { getWorkouts } from '$lib/api';
	import { browser } from '$app/environment';
	import { logger } from '$lib/logger';
	import SetPillVariants from '$lib/components/ui/SetPillVariants.svelte';

	function getStoredString(key: string, fallback: string): string {
		if (!browser) return fallback;
		return localStorage.getItem(key) ?? fallback;
	}

	function getStoredNumber(key: string, fallback: number): number {
		if (!browser) return fallback;
		const raw = localStorage.getItem(key);
		if (raw === null) return fallback;
		const parsed = Number(raw);
		return Number.isFinite(parsed) ? parsed : fallback;
	}

	function normalizeSetChipStyle(n: number): 1 | 2 | 3 | 4 | 5 {
		if (n === 1 || n === 2 || n === 3 || n === 4 || n === 5) return n;
		return 1;
	}

	// Settings stores
	const unitPreference = writable(getStoredString('unitPreference', 'kg'));
	const restTimer = writable(getStoredNumber('restTimer', 90));
	const autoEndTimeout = writable(getStoredNumber('autoEndTimeout', 300));
	const viewDensity = writable(getStoredString('viewDensity', 'comfortable'));
	const accentColor = writable(getStoredString('accentColor', '#652B26'));
	const setChipStyle = writable(normalizeSetChipStyle(getStoredNumber('setChipStyle', 1)));

	// Save settings to localStorage when they change
	$: {
		if (browser) {
			localStorage.setItem('unitPreference', $unitPreference);
			localStorage.setItem('restTimer', $restTimer.toString());
			localStorage.setItem('autoEndTimeout', $autoEndTimeout.toString());
			localStorage.setItem('viewDensity', $viewDensity);
			localStorage.setItem('accentColor', $accentColor);
			localStorage.setItem('setChipStyle', $setChipStyle.toString());
		}
	}

	async function exportData() {
		try {
			const data = await getWorkouts();
			const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
			const url = window.URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `swolemate-backup-${new Date().toISOString().split('T')[0]}.json`;
			a.click();
			window.URL.revokeObjectURL(url);
		} catch (error) {
			logger.error('settings', 'Failed to export data', { error });
		}
	}
</script>

<div class="container mx-auto p-4 space-y-8">
	<header class="text-center">
		<h1 class="h1 mb-4">Settings</h1>
	</header>

	<Tabs defaultValue="workout">
		<Tabs.List class="flex gap-2 flex-wrap">
			<Tabs.Trigger class="btn variant-ghost-primary" value="workout">
				<span class="text-xl mr-2">💪</span> Workout
			</Tabs.Trigger>
			<Tabs.Trigger class="btn variant-ghost-primary" value="appearance">
				<span class="text-xl mr-2">🎨</span> Appearance
			</Tabs.Trigger>
			<Tabs.Trigger class="btn variant-ghost-primary" value="data">
				<span class="text-xl mr-2">💾</span> Data
			</Tabs.Trigger>
			<Tabs.Trigger class="btn variant-ghost-primary" value="notifications">
				<span class="text-xl mr-2">🔔</span> Notifications
			</Tabs.Trigger>
			<Tabs.Indicator />
		</Tabs.List>

		<Tabs.Content value="workout">
			<div class="card variant-glass-surface p-4 space-y-4">
				<div class="space-y-2">
					<label class="label">
						<span>Weight Unit</span>
						<select class="select" bind:value={$unitPreference}>
							<option value="kg">Kilograms (kg)</option>
							<option value="lbs">Pounds (lbs)</option>
						</select>
					</label>

					<label class="label">
						<span>Rest Timer Duration (seconds)</span>
						<input type="number" class="input" bind:value={$restTimer} min="0" max="300" />
					</label>

					<label class="label">
						<span>Auto-end Workout Timeout (minutes)</span>
						<input type="number" class="input" bind:value={$autoEndTimeout} min="0" max="60" />
					</label>
				</div>
			</div>
		</Tabs.Content>

		<Tabs.Content value="appearance">
			<div class="card variant-glass-surface p-4 space-y-4">
				<div class="space-y-2">
					<label class="label">
						<span>View Density</span>
						<select class="select" bind:value={$viewDensity}>
							<option value="comfortable">Comfortable</option>
							<option value="compact">Compact</option>
						</select>
					</label>

					<label class="label">
						<span>Accent Color</span>
						<input type="color" class="input" bind:value={$accentColor} />
					</label>

					<label class="label">
						<span>Set Chip Style</span>
						<select class="select" bind:value={$setChipStyle}>
							<option value={1}>1) Segmented pill</option>
							<option value={5}>5) Segmented + weight intensity</option>
							<option value={2}>2) Chip + count badge</option>
							<option value={3}>3) Weight-intensity scale</option>
							<option value={4}>4) Two-row reps + weight</option>
						</select>
					</label>
				</div>

				<SetPillVariants
					selectedOption={$setChipStyle}
					on:select={(e) => setChipStyle.set(e.detail.option)}
				/>
			</div>
		</Tabs.Content>

		<Tabs.Content value="data">
			<div class="card variant-glass-surface p-4 space-y-4">
				<div class="grid gap-4">
					<button class="btn variant-filled-primary" on:click={exportData}>
						<span class="text-xl mr-2">📤</span> Export Workout Data
					</button>

					<button class="btn variant-filled-surface">
						<span class="text-xl mr-2">📥</span> Import Workout Data
					</button>

					<button class="btn variant-filled-error">
						<span class="text-xl mr-2">🗑️</span> Clear All Data
					</button>
				</div>
			</div>
		</Tabs.Content>

		<Tabs.Content value="notifications">
			<div class="card variant-glass-surface p-4 space-y-4">
				<div class="space-y-2">
					<label class="label">
						<span>Workout Reminders</span>
						<select class="select">
							<option value="none">None</option>
							<option value="daily">Daily</option>
							<option value="weekly">Weekly</option>
						</select>
					</label>

					<label class="label">
						<span>Rest Timer Notifications</span>
						<div class="flex items-center space-x-2">
							<input type="checkbox" class="checkbox" />
							<span>Enable sound</span>
						</div>
					</label>

					<label class="label">
						<span>Progress Milestones</span>
						<div class="flex items-center space-x-2">
							<input type="checkbox" class="checkbox" />
							<span>Show notifications</span>
						</div>
					</label>
				</div>
			</div>
		</Tabs.Content>
	</Tabs>
</div>

<style>
	.label {
		display: block;
	}
	.label > span:first-child {
		font-weight: 700;
	}
	.select,
	.input {
		width: 100%;
	}
</style>
