<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ImageLike } from '@flare/protocol';
	import Icon from '$lib/components/Icon.svelte';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import { Button, type ButtonProps } from '$lib/components/ui/button';
	import type { Toast as ToastType } from '$lib/ui.svelte';
	import Toast from './Toast.svelte';
	import ActionMenu from './ActionMenu.svelte';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import KeyboardShortcut from '$lib/components/KeyboardShortcut.svelte';
	import type { ActionDefinition } from './actions';
	import { keyEventMatches } from '$lib/props';

	type Props = {
		title?: string | Snippet;
		icon?: ImageLike | null;
		actions?: ActionDefinition[];
		/** Optional override to render the primary action button. Prefer not providing if possible. */
		primaryAction?: Snippet<[{ props: ButtonProps }]>;

		toast?: ToastType | null;
		onToastAction?: (toastId: number, actionType: 'primary' | 'secondary') => void;
	};

	let {
		title,
		icon,
		actions,
		toast = null,
		onToastAction,
		primaryAction: primaryActionOverride
	}: Props = $props();

	const primaryAction = $derived(actions?.[0] ?? null);

	const handleKeydown = (event: KeyboardEvent) => {
		if (event.key === 'Enter') {
			if (
				event.target instanceof HTMLElement &&
				event.target.closest('[data-slot="dropdown-menu-content"]')
			) {
				return;
			}
			event.preventDefault();

			if (!actions?.[0]?.shortcut && !event.ctrlKey && !event.metaKey && !event.shiftKey) {
				actions?.[0]?.handler?.();
				return;
			} else if (!actions?.[1]?.shortcut && event.ctrlKey && !event.metaKey && !event.shiftKey) {
				actions?.[1]?.handler?.();
				return;
			}
		}

		for (const action of actions ?? []) {
			if (action.shortcut && keyEventMatches(event, action.shortcut)) {
				action.handler?.();
				return;
			}
		}
	};
</script>

<svelte:document onkeydown={handleKeydown} />

<footer class="bg-card flex h-10 shrink-0 items-center border-t px-2">
	{#if toast}
		<Toast {toast} {onToastAction} />
	{:else if title || icon}
		<div class="flex min-w-0 items-center gap-2.5 pl-1">
			{#if icon}
				<Icon {icon} class="size-5 shrink-0" />
			{/if}
			{#if title}
				{#if typeof title === 'string'}
					<span class="text-muted-foreground truncate text-sm">{title}</span>
				{:else}
					{@render title()}
				{/if}
			{/if}
		</div>
	{/if}

	<div class="ml-auto flex items-center">
		{#if primaryActionOverride}
			{@render primaryActionOverride({ props: { variant: 'ghost', size: 'action' } })}
		{:else if primaryAction}
			<div class="peer order-1">
				<Button
					variant="ghost"
					size="action"
					class={primaryAction.style === 'destructive' ? 'text-destructive' : ''}
					onclick={primaryAction.handler}
					disabled={primaryAction.disabled}
				>
					{primaryAction.title}

					<KeyboardShortcut shortcut={{ modifiers: [], key: 'enter' }} />
				</Button>
			</div>
		{/if}
		{#if actions && actions.length > 1}
			<div class="peer order-3">
				<ActionMenu>
					{#each actions as action, i}
						<DropdownMenu.Item
							class="rounded-md p-2 text-left {action.style === 'destructive'
								? 'text-destructive focus:text-destructive-foreground focus:bg-destructive'
								: ''}"
							disabled={action.disabled}
							onclick={action.handler}
						>
							{#if action.icon}
								<Icon icon={action.icon} class="size-4" />
							{/if}
							{action.title}
							{#if i == 0}
								<DropdownMenu.Shortcut>
									<KeyboardShortcut shortcut={{ key: 'enter', modifiers: [] }} />
								</DropdownMenu.Shortcut>
							{:else if i == 1}
								<DropdownMenu.Shortcut>
									<KeyboardShortcut shortcut={{ key: 'enter', modifiers: ['ctrl'] }} />
								</DropdownMenu.Shortcut>
							{:else if action.shortcut}
								<DropdownMenu.Shortcut>
									<KeyboardShortcut shortcut={action.shortcut} />
								</DropdownMenu.Shortcut>
							{/if}
						</DropdownMenu.Item>
					{/each}
				</ActionMenu>
			</div>
			{#if primaryAction || primaryActionOverride}
				<Separator
					orientation="vertical"
					class="order-2 mr-1 ml-2 !h-3 !w-[2px] transition-opacity peer-hover:opacity-0"
				/>
			{/if}
		{/if}
	</div>
</footer>
