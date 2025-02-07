<script lang="ts">
	import { TabGroup, Tab } from '@skeletonlabs/skeleton';
	import { getContext, setContext } from 'svelte';
	import { writable } from 'svelte/store';

	// Settings stores
	const unitPreference = writable(localStorage.getItem('unitPreference') || 'kg');
	const restTimer = writable(Number(localStorage.getItem('restTimer')) || 90);
	const autoEndTimeout = writable(Number(localStorage.getItem('autoEndTimeout')) || 300);
	const viewDensity = writable(localStorage.getItem('viewDensity') || 'comfortable');
	const accentColor = writable(localStorage.getItem('accentColor') || '#652B26');

	// Save settings to localStorage when they change
	$: {
		localStorage.setItem('unitPreference', $unitPreference);
		localStorage.setItem('restTimer', $restTimer.toString());
		localStorage.setItem('autoEndTimeout', $autoEndTimeout.toString());
		localStorage.setItem('viewDensity', $viewDensity);
		localStorage.setItem('accentColor', $accentColor);
	}

	let tabSet = 0;

	async function exportData() {
		try {
			const response = await fetch('/api/workouts');
			const data = await response.json();
			const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
			const url = window.URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `swolemate-backup-${new Date().toISOString().split('T')[0]}.json`;
			a.click();
			window.URL.revokeObjectURL(url);
		} catch (error) {
			console.error('Failed to export data:', error);
		}
	}
</script>

<div class="container mx-auto p-4 space-y-8">
	<header class="text-center">
		<h1 class="h1 mb-4">Settings</h1>
	</header>

	<TabGroup>
		<Tab bind:group={tabSet} name="workout" value={0}>
			<span class="text-xl mr-2">💪</span> Workout
		</Tab>
		<Tab bind:group={tabSet} name="appearance" value={1}>
			<span class="text-xl mr-2">🎨</span> Appearance
		</Tab>
		<Tab bind:group={tabSet} name="data" value={2}>
			<span class="text-xl mr-2">💾</span> Data
		</Tab>
		<Tab bind:group={tabSet} name="notifications" value={3}>
			<span class="text-xl mr-2">🔔</span> Notifications
		</Tab>

		<!-- Tab Panels -->
		<svelte:fragment slot="panel">
			{#if tabSet === 0}
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
							<input 
								type="number" 
								class="input" 
								bind:value={$restTimer}
								min="0"
								max="300"
							/>
						</label>

						<label class="label">
							<span>Auto-end Workout Timeout (minutes)</span>
							<input 
								type="number" 
								class="input" 
								bind:value={$autoEndTimeout}
								min="0"
								max="60"
							/>
						</label>
					</div>
				</div>
			{:else if tabSet === 1}
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
							<input 
								type="color" 
								class="input" 
								bind:value={$accentColor}
							/>
						</label>
					</div>
				</div>
			{:else if tabSet === 2}
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
			{:else if tabSet === 3}
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
			{/if}
		</svelte:fragment>
	</TabGroup>
</div>

<style lang="postcss">
	.label {
		@apply block space-y-2;
	}
	.label > span:first-child {
		@apply font-bold;
	}
	.select, .input {
		@apply w-full;
	}
</style> 