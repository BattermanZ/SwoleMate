<script lang="ts">
	import type { ProgressOverview } from '$lib/types';
	import ProgressOverviewPanel from '$lib/components/progress/ProgressOverview.svelte';
	import RecentPrFeed from '$lib/components/progress/RecentPrFeed.svelte';

	export let progressOverview: ProgressOverview | null = null;
	export let loadingOverall = false;
	export let errorOverall: string | null = null;
</script>

<div class="space-y-4">
	<ProgressOverviewPanel
		overview={progressOverview}
		loading={loadingOverall}
		error={errorOverall}
	/>
	<RecentPrFeed
		prs={progressOverview?.recent_prs ?? []}
		recentBests={progressOverview?.recent_bests ?? []}
	/>
</div>
