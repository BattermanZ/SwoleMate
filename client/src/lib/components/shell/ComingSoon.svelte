<script lang="ts">
	import { PageHero } from '$lib/components/ui';
	interface Props {
		title: string;
		blurb?: string;
	}
	let { title, blurb }: Props = $props();
</script>

<PageHero kicker="► Coming soon">
	{#snippet title()}{title} <em>— rebuilding.</em>{/snippet}
	{#snippet sub()}
		{#if blurb}{blurb}{:else}This route is being ported to the new design system. Check back soon.{/if}
	{/snippet}
</PageHero>
