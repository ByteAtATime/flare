<script lang="ts">
	type Props = {
		title: string;
		description?: string;
		children: import('svelte').Snippet;
	};

	let { title, description, children }: Props = $props();
</script>

<div class="mb-6 border-b pb-6">
	<div class="mb-4">
		<h3 class="text-base font-semibold">{title}</h3>
		{#if description}
			<p class="text-muted-foreground mt-1 text-sm">{description}</p>
		{/if}
	</div>
	<div class="space-y-4">
		{@render children()}
	</div>
</div>
