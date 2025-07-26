<script lang="ts">
	import type { ImageLike } from '@flare/protocol';
	import { resolveIcon } from '$lib/assets';
	import icons from '$lib/icons.svg';
	import { getContext, hasContext } from 'svelte';
	import { mode } from 'mode-watcher';
	import { colorLikeToColor } from '$lib/props/color';

	type Props = {
		icon: ImageLike | undefined | null;
		class?: string;
		assetsPath?: string;
	};
	let { icon, class: className, assetsPath: propAssetsPath }: Props = $props();

	const assetsPath = $derived.by(
		propAssetsPath
			? () => propAssetsPath
			: hasContext('assetsPath')
				? getContext<() => string>('assetsPath')
				: () => ''
	);
	const iconInfo = $derived(resolveIcon(icon, assetsPath));

	const style = $derived.by(() => {
		if (!iconInfo) return '';

		let styles = '';

		if (iconInfo.type === 'image' && iconInfo.mask) {
			if (iconInfo.mask === 'circle') {
				styles += 'border-radius: 50%;';
			} else if (iconInfo.mask === 'roundedRectangle') {
				styles += 'border-radius: 0.375rem;';
			}
		}

		if ('tintColor' in iconInfo && iconInfo.tintColor) {
			const color = colorLikeToColor(iconInfo.tintColor, mode.current === 'dark');

			if (iconInfo.type === 'raycast') {
				return `color: ${color};`;
			}
			if (iconInfo.type === 'image') {
				styles += ` background-color: ${color}; -webkit-mask-image: url(${iconInfo.src}); mask-image: url(${iconInfo.src}); -webkit-mask-size: contain; mask-size: contain; -webkit-mask-repeat: no-repeat; mask-repeat: no-repeat; -webkit-mask-position: center; mask-position: center;`;
			}
		}

		return styles;
	});
</script>

{#if iconInfo}
	{#if iconInfo.type === 'raycast'}
		<svg class="size-4 shrink-0 fill-none {className ?? ''}" {style}>
			<use href="{icons}#{iconInfo.name}"></use>
		</svg>
	{:else if iconInfo.type === 'image'}
		{#if iconInfo.tintColor}
			<div class="size-4 shrink-0 {className ?? ''}" {style}></div>
		{:else}
			<img
				src={iconInfo.src}
				alt=""
				class="size-4 shrink-0 object-contain {className ?? ''}"
				{style}
			/>
		{/if}
	{:else if iconInfo.type === 'emoji'}
		<span class="shrink-0 {className ?? ''}">{iconInfo.emoji}</span>
	{/if}
{/if}
