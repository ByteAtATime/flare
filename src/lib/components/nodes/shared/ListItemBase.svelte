<script lang="ts">
	import Icon from '$lib/components/Icon.svelte';
	import type { ImageLike } from '@flare/protocol';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import type { Snippet } from 'svelte';

	type Props = {
		title: string;
		subtitle?: string | null;
		icon?: ImageLike | null;
		isSelected: boolean;
		accessories?: Snippet;
		assetsPath?: string;
	} & HTMLButtonAttributes;

	let { title, subtitle, icon, isSelected, accessories, assetsPath, ...restProps }: Props =
		$props();
</script>

<button
	type="button"
	class="hover:bg-accent/50 flex h-12 w-full items-center gap-3 rounded-md px-2 text-left"
	class:!bg-accent={isSelected}
	data-testid="list-item"
	{...restProps}
>
	{#if icon}
		<Icon {icon} {assetsPath} class="size-[22px]" />
	{:else}
		<div class="size-[22px]"></div>
	{/if}

	<div class="flex flex-grow items-baseline gap-3 overflow-hidden">
		<p class="whitespace-nowrap">{title}</p>
		{#if subtitle}
			<p class="text-muted-foreground truncate">{subtitle}</p>
		{/if}
	</div>

	{#if accessories}
		<div class="ml-auto flex shrink-0 items-center gap-4">
			{@render accessories()}
		</div>
	{/if}
</button>
