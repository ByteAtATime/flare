<script lang="ts">
	type Props = {
		label: string;
		description?: string;
		control: import('svelte').Snippet;
		layout?: 'inline' | 'stacked';
	};

	let { label, description, control, layout = 'inline' }: Props = $props();
</script>

<div class="flex" class:flex-col={layout === 'stacked'} class:items-start={layout === 'inline'}>
	<div class="flex-1" class:mb-2={layout === 'stacked'}>
		<div class="text-sm font-medium">{label}</div>
		{#if description}
			<p class="text-muted-foreground mt-0.5 text-xs">{description}</p>
		{/if}
	</div>
	<div
		class="flex items-center"
		class:ml-4={layout === 'inline'}
		class:w-full={layout === 'stacked'}
	>
		{@render control()}
	</div>
</div>
