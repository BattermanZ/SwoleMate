<script lang="ts">
	interface Props {
		size?: number;
		radius?: number;
		fontSize?: number;
		'aria-label'?: string;
	}
	let {
		size = 30,
		radius,
		fontSize,
		'aria-label': ariaLabel = 'SwoleMate logo'
	}: Props = $props();
	let r = $derived(radius ?? Math.round(size * 0.3));
	let fs = $derived(fontSize ?? Math.round(size * 0.43));
</script>

<span
	class="logo"
	role="img"
	aria-label={ariaLabel}
	style="width:{size}px;height:{size}px;border-radius:{r}px;font-size:{fs}px;"
>
	<span aria-hidden="true">SM</span>
</span>

<style>
	.logo {
		flex: none;
		display: grid;
		place-items: center;
		background: linear-gradient(135deg, var(--clay-2), var(--clay));
		color: white;
		font:
			800 1em/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.02em;
		box-shadow: 0 4px 10px -3px rgba(255, 94, 31, 0.55);
	}
</style>
