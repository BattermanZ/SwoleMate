<script lang="ts">
	import type { WorkoutStats } from '$lib/types';
	import OverallCharts from '$lib/components/progress/OverallCharts.svelte';

	export let workoutStats: WorkoutStats | null = null;
</script>

<OverallCharts {workoutStats} />
