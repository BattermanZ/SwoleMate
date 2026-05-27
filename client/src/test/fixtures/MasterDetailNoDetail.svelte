<script lang="ts">
	import MasterDetail from '$lib/components/ui/MasterDetail.svelte';
</script>

<MasterDetail>
	{#snippet list()}
		<div data-testid="list-content">LIST</div>
	{/snippet}
	{#snippet empty()}
		<div data-testid="empty-content">Nothing selected</div>
	{/snippet}
</MasterDetail>
