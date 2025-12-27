<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		header: Snippet;
		content: Snippet;
		footer: Snippet;
	};
	let { header, content, footer }: Props = $props();
</script>

<main
	class="bg-background text-foreground flex h-screen flex-col"
	style="
		box-shadow: 
			inset 0 0 0 2px rgba(255, 255, 255, 0.6),
			0 0 0 1px rgba(255, 255, 255, 0.5),
			0 20px 60px rgba(0, 0, 0, 0.9),
			0 10px 30px rgba(0, 0, 0, 0.7);
		border-radius: 8px;
	"
>
	{@render header()}
	{@render content()}
	{@render footer()}
</main>
