<script lang="ts">
	import type { PluginInfo } from '@flare/protocol';
	import Calculator from '$lib/components/Calculator.svelte';
	import BaseList from '$lib/components/BaseList.svelte';
	import ListItemBase from '../nodes/shared/ListItemBase.svelte';
	import path from 'path';
	import { tick } from 'svelte';
	import type { Quicklink } from '$lib/quicklinks.svelte';
	import { appsStore } from '$lib/apps.svelte';
	import { frecencyStore } from '$lib/frecency.svelte';
	import { quicklinksStore } from '$lib/quicklinks.svelte';
	import { useCommandPaletteItems, useCommandPaletteActions } from '$lib/command-palette.svelte';
	import CommandPaletteActionBar from './ActionBar.svelte';
	import { focusManager } from '$lib/focus.svelte';
	import HeaderInput from '../HeaderInput.svelte';
	import { Input } from '$lib/components/ui/input';
	import MainLayout from '../layout/MainLayout.svelte';
	import Header from '../layout/Header.svelte';

	type Props = {
		plugins: PluginInfo[];
		onRunPlugin: (plugin: PluginInfo) => void;
	};

	let { plugins, onRunPlugin }: Props = $props();

	const { apps: installedApps } = $derived(appsStore);
	const { quicklinks } = $derived(quicklinksStore);
	const { data: frecencyData } = $derived(frecencyStore);

	let searchText = $state('');
	let quicklinkArgument = $state('');
	let selectedIndex = $state(0);
	let listElement: HTMLElement | null = $state(null);
	let searchInputEl: HTMLInputElement | null = $state(null);
	let argumentInputEl: HTMLInputElement | null = $state(null);
	let selectedQuicklinkForArgument: Quicklink | null = $state(null);

	const { displayItems } = $derived.by(
		useCommandPaletteItems({
			searchText: () => searchText,
			plugins: () => plugins,
			installedApps: () => installedApps,
			quicklinks: () => quicklinks,
			frecencyData: () => frecencyData,
			selectedQuicklinkForArgument: () => selectedQuicklinkForArgument
		})
	);

	const selectedItem = $derived(displayItems[selectedIndex]);

	$effect(() => {
		if (focusManager.activeScope === 'main-input') {
			tick().then(() => {
				searchInputEl?.focus();
			});
		}
	});

	$effect(() => {
		if (focusManager.activeScope === 'quicklink-argument') {
			tick().then(() => {
				argumentInputEl?.focus();
			});
		}
	});

	function resetState() {
		searchText = '';
		quicklinkArgument = '';
		selectedIndex = 0;
		selectedQuicklinkForArgument = null;
	}

	function focusArgumentInput() {
		focusManager.requestFocus('quicklink-argument');
	}

	async function setSearchText(text: string) {
		searchText = text;
	}

	const actions = useCommandPaletteActions({
		selectedItem: () => selectedItem,
		onRunPlugin,
		resetState,
		focusArgumentInput
	});

	$effect(() => {
		const item = displayItems[selectedIndex];
		if (item?.type === 'quicklink' && item.data.link.includes('{argument}')) {
			selectedQuicklinkForArgument = item.data;
		} else {
			console.log('null haha');
			selectedQuicklinkForArgument = null;
		}
	});

	$effect(() => {
		if (!selectedQuicklinkForArgument) {
			focusManager.releaseFocus('quicklink-argument');
		}
	});

	async function handleArgumentKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			if (selectedQuicklinkForArgument) {
				await actions.executeQuicklink(selectedQuicklinkForArgument, quicklinkArgument);
			}
		} else if (e.key === 'Escape' || (e.key === 'Backspace' && quicklinkArgument === '')) {
			e.preventDefault();
			focusManager.releaseFocus('quicklink-argument');
		}
	}
</script>

<MainLayout>
	{#snippet header()}
		<Header>
			<div class="relative flex w-full items-center">
				<HeaderInput
					placeholder={selectedQuicklinkForArgument
						? selectedQuicklinkForArgument.name
						: 'Search for apps and commands...'}
					bind:value={searchText}
					bind:ref={searchInputEl}
					autofocus
					class="!pl-0"
				/>

				{#if selectedQuicklinkForArgument}
					<div
						class="pointer-events-none absolute top-0 left-0 flex h-full w-full items-center pl-4"
					>
						<span class="text-lg whitespace-pre text-transparent"
							>{searchText || selectedQuicklinkForArgument.name}</span
						>
						<span class="w-2"></span>
						<div class="pointer-events-auto">
							<div class="inline-grid items-center">
								<span
									class="invisible col-start-1 row-start-1 px-3 text-base whitespace-pre md:text-sm"
									aria-hidden="true"
								>
									{quicklinkArgument || 'Query'}
								</span>

								<Input
									class="border-border col-start-1 row-start-1 h-7 w-full"
									placeholder="Query"
									bind:value={quicklinkArgument}
									bind:ref={argumentInputEl}
									onkeydown={handleArgumentKeydown}
								/>
							</div>
						</div>
					</div>
				{/if}
			</div>
		</Header>
	{/snippet}

	{#snippet content()}
		<div class="grow overflow-y-auto" data-testid="command-palette-content">
			<BaseList
				items={displayItems.map((item) => ({ ...item, itemType: 'item' }))}
				onenter={actions.handleEnter}
				bind:selectedIndex
				bind:listElement
			>
				{#snippet itemSnippet({ item, isSelected, onclick })}
					{#if item.type === 'calculator'}
						<Calculator
							searchText={item.data.value}
							mathResult={item.data.result}
							mathResultType={item.data.resultType}
							{isSelected}
							onSelect={onclick}
						/>
					{:else if item.type === 'plugin'}
						{@const assetsPath = path.dirname(item.data.pluginPath) + '/assets'}
						<ListItemBase
							title={item.data.title}
							subtitle={item.data.pluginTitle}
							icon={item.data.icon || 'app-window-16'}
							{assetsPath}
							{isSelected}
							{onclick}
						>
							{#snippet accessories()}
								<span class="text-muted-foreground ml-auto text-xs whitespace-nowrap">
									Command
								</span>
							{/snippet}
						</ListItemBase>
					{:else if item.type === 'app'}
						<ListItemBase
							title={item.data.name}
							subtitle={item.data.comment}
							icon={item.data.icon_path ?? 'app-window-16'}
							{isSelected}
							{onclick}
						>
							{#snippet accessories()}
								<span class="text-muted-foreground ml-auto text-xs whitespace-nowrap">
									Application
								</span>
							{/snippet}
						</ListItemBase>
					{:else if item.type === 'quicklink'}
						<ListItemBase
							title={item.data.name}
							subtitle={item.data.link.replace(/\{argument\}/g, '...')}
							icon={item.data.icon ?? 'link-16'}
							{isSelected}
							{onclick}
						>
							{#snippet accessories()}
								<span class="text-muted-foreground ml-auto text-xs whitespace-nowrap">
									Quicklink
								</span>
							{/snippet}
						</ListItemBase>
					{/if}
				{/snippet}
			</BaseList>
		</div>
	{/snippet}

	{#snippet footer()}
		<CommandPaletteActionBar {selectedItem} {actions} {setSearchText} />
	{/snippet}
</MainLayout>
