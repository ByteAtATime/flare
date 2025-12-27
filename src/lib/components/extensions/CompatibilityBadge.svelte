<script lang="ts">
	import Icon from '../Icon.svelte';
	import { cn } from '$lib/utils';

	type Props = {
		score: number;
		size?: 'sm' | 'md' | 'lg';
		showLabel?: boolean;
		class?: string;
	};

	let { score, size = 'md', showLabel = false, class: className }: Props = $props();

	const sizeClasses = {
		sm: 'h-4 gap-1 px-1.5 py-0.5 text-[10px]',
		md: 'h-5 gap-1.5 px-2 py-1 text-xs',
		lg: 'h-6 gap-2 px-2.5 py-1 text-sm'
	};

	const iconSizeClasses = {
		sm: 'size-3',
		md: 'size-3.5',
		lg: 'size-4'
	};

	const badge = $derived.by(() => {
		if (score >= 90) {
			return {
				label: 'Excellent',
				description: 'Fully compatible with Linux',
				color: 'bg-green-900/50 text-green-300 border border-green-700/50',
				icon: 'check-circle-16'
			};
		} else if (score >= 70) {
			return {
				label: 'Good',
				description: 'Good Linux compatibility with minor limitations',
				color: 'bg-yellow-900/50 text-yellow-300 border border-yellow-700/50',
				icon: 'warning-16'
			};
		} else if (score >= 50) {
			return {
				label: 'Limited',
				description: 'Limited Linux compatibility, may have issues',
				color: 'bg-orange-900/50 text-orange-300 border border-orange-700/50',
				icon: 'warning-16'
			};
		} else {
			return {
				label: 'Incompatible',
				description: 'Likely incompatible with Linux',
				color: 'bg-red-900/50 text-red-300 border border-red-700/50',
				icon: 'x-mark-circle-16'
			};
		}
	});
</script>

<div
	class={cn(
		'inline-flex items-center rounded-full font-medium',
		sizeClasses[size],
		badge.color,
		className
	)}
	title={`${badge.description} (${score}/100)`}
>
	<Icon icon={badge.icon} class={iconSizeClasses[size]} />
	{#if showLabel}
		<span>{badge.label}</span>
	{/if}
</div>
