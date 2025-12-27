<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount, tick } from 'svelte';
	import BaseList from './BaseList.svelte';
	import ListItemBase from './nodes/shared/ListItemBase.svelte';
	import MainLayout from './layout/MainLayout.svelte';
	import Header from './layout/Header.svelte';
	import HeaderInput from './HeaderInput.svelte';

	let items: string[] = $state([]);
	let searchText = $state('');
	let prompt = $state('');
	let caseInsensitive = $state(false);
	let selectedIndex = $state(0);
	let listElement: HTMLElement | null = $state(null);
	let searchInputEl: HTMLInputElement | null = $state(null);

	const filteredItems = $derived(() => {
		if (!searchText) {
			return items;
		}
		const query = caseInsensitive ? searchText.toLowerCase() : searchText;
		return items.filter((item) => {
			const itemText = caseInsensitive ? item.toLowerCase() : item;
			return itemText.includes(query);
		});
	});

	const displayItems = $derived(
		filteredItems().map((item, i) => ({
			id: `dmenu-${i}`,
			type: 'dmenu' as const,
			data: item,
			itemType: 'item' as const
		}))
	);

	onMount(async () => {
		// Load dmenu data from Rust backend
		const [itemsData, promptData, caseInsensitiveData] = await Promise.all([
			invoke<string[]>('dmenu_get_items'),
			invoke<string>('dmenu_get_prompt'),
			invoke<boolean>('dmenu_get_case_insensitive')
		]);
		items = itemsData;
		prompt = promptData;
		caseInsensitive = caseInsensitiveData;

		// Focus the input after mounting
		await tick();
		searchInputEl?.focus();
	});

	async function handleSelect(item: string) {
		await invoke('dmenu_select_item', { item });
	}

	async function handleCancel() {
		await invoke('dmenu_cancel');
	}

	function handleEnter(item: { data: string }) {
		handleSelect(item.data);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			handleCancel();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<MainLayout>
	{#snippet header()}
		<Header>
			<HeaderInput
				placeholder={prompt || 'Select an option...'}
				bind:value={searchText}
				bind:ref={searchInputEl}
			/>
		</Header>
	{/snippet}

	{#snippet content()}
		<div class="grow overflow-y-auto" data-testid="dmenu-content">
			<BaseList items={displayItems} onenter={handleEnter} bind:selectedIndex bind:listElement>
				{#snippet itemSnippet({ item, isSelected, onclick })}
					<ListItemBase title={item.data} icon="list-16" {isSelected} {onclick} />
				{/snippet}
			</BaseList>
		</div>
	{/snippet}

	{#snippet footer()}
		<div class="text-muted-foreground/50 absolute right-0 bottom-2 left-0 text-center text-[10px]">
			{items.length} items · dmenu mode
		</div>
	{/snippet}
</MainLayout>
