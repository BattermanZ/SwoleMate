<script lang="ts">
	import '../app.css';

	let { children } = $props();
</script>

<svelte:head>
	<title>SwoleMate</title>
</svelte:head>

{@render children?.()}
