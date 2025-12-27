<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount, untrack } from 'svelte';
	import { Loader2 } from '@lucide/svelte';
	import ListItemBase from './nodes/shared/ListItemBase.svelte';
	import ActionBar from './nodes/shared/ActionBar.svelte';
	import BaseList from './BaseList.svelte';
	import HeaderInput from './HeaderInput.svelte';
	import MainLayout from './layout/MainLayout.svelte';
	import Header from './layout/Header.svelte';
	import InfoList from './InfoList.svelte';
	import type { ActionDefinition } from './nodes/shared/actions';
	import snippetIcon from '$lib/assets/snippets-package-1616x16@2x.png?inline';

	type Props = {
		onBack: () => void;
		onEdit: (snippet: Snippet) => void;
	};

	type Snippet = {
		id: number;
		name: string;
		keyword: string;
		content: string;
		createdAt: string;
		updatedAt: string;
		timesUsed: number;
		lastUsedAt: string;
	};

	type DisplayItem = {
		id: number | string;
		itemType: 'item' | 'header';
		data: Snippet | string;
	};

	let { onBack, onEdit }: Props = $props();

	let snippets = $state<Snippet[]>([]);
	let selectedIndex = $state(0);
	let searchText = $state('');
	let isFetching = $state(false);

	const displayedItems = $derived.by(() => {
		const items: DisplayItem[] = [];
		let lastHeader = '';

		const today = new Date();
		today.setHours(0, 0, 0, 0);
		const yesterday = new Date(today);
		yesterday.setDate(yesterday.getDate() - 1);

		for (const snippet of snippets) {
			const snippetDate = new Date(snippet.updatedAt);

			let header = '';
			const snippetDay = new Date(snippetDate);
			snippetDay.setHours(0, 0, 0, 0);

			if (snippetDay.getTime() === today.getTime()) {
				header = 'Today';
			} else if (snippetDay.getTime() === yesterday.getTime()) {
				header = 'Yesterday';
			} else {
				header = snippetDay.toLocaleDateString(undefined, {
					year: 'numeric',
					month: 'long'
				});
			}

			if (header !== lastHeader) {
				items.push({ id: `header-${header}`, itemType: 'header', data: header });
				lastHeader = header;
			}
			items.push({ id: snippet.id, itemType: 'item', data: snippet });
		}
		return items;
	});

	const selectedItem = $derived(
		displayedItems[selectedIndex]?.itemType === 'item'
			? (displayedItems[selectedIndex].data as Snippet)
			: null
	);

	const fetchSnippets = async () => {
		if (isFetching) return;
		isFetching = true;
		try {
			const newItems = await invoke<Snippet[]>('list_snippets', {
				searchTerm: searchText || null
			});
			snippets = newItems;
			if (selectedIndex >= newItems.length) {
				selectedIndex = 0;
			}
		} catch (e) {
			console.error('Failed to fetch snippets:', e);
		} finally {
			isFetching = false;
		}
	};

	const formatDateTime = (dateString: string) => {
		const date = new Date(dateString);
		if (date.getFullYear() < 1971) return 'Never';
		return `Today at ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}`;
	};

	const handlePaste = async (item: Snippet) => {
		await invoke('paste_snippet_content', { content: item.content });
		await invoke('snippet_was_used', { id: item.id });
		const updatedItems = snippets.map((i) =>
			i.id === item.id
				? {
						...i,
						timesUsed: i.timesUsed + 1,
						lastUsedAt: new Date().toISOString()
					}
				: i
		);
		snippets = updatedItems;
		onBack();
	};

	const handleDelete = async (item: Snippet) => {
		await invoke('delete_snippet', { id: item.id });
		fetchSnippets();
	};

	onMount(() => {
		fetchSnippets();
	});

	$effect(() => {
		const term = searchText;
		untrack(() => {
			const timer = setTimeout(() => {
				if (term === searchText) {
					fetchSnippets();
				}
			}, 300);
			return () => clearTimeout(timer);
		});
	});

	const actions: ActionDefinition[] = $derived(
		selectedItem
			? [
					{
						title: 'Paste',
						handler: () => handlePaste(selectedItem)
					},
					{
						title: 'Edit',
						shortcut: { key: 'e', modifiers: ['cmd'] },
						handler: () => onEdit(selectedItem)
					},
					{
						title: 'Delete',
						shortcut: { key: 'x', modifiers: ['ctrl'] },
						handler: () => handleDelete(selectedItem)
					}
				]
			: []
	);
</script>

<MainLayout>
	{#snippet header()}
		<Header showBackButton={true} onPopView={onBack}>
			<HeaderInput
				placeholder="Search snippets..."
				bind:value={searchText}
				autofocus
				class="!pl-2.5"
			/>
		</Header>
	{/snippet}
	{#snippet content()}
		<div class="grid grow grid-cols-[minmax(0,_1.5fr)_minmax(0,_2.5fr)] overflow-y-hidden">
			<div class="flex-grow overflow-y-auto border-r">
				{#if isFetching && snippets.length === 0}
					<div class="text-muted-foreground flex h-full items-center justify-center">
						<Loader2 class="size-6 animate-spin" />
					</div>
				{:else}
					<BaseList
						items={displayedItems}
						bind:selectedIndex
						onenter={(item) => handlePaste(item.data as Snippet)}
						isItemSelectable={(item) => item.itemType === 'item'}
					>
						{#snippet itemSnippet({ item, isSelected, onclick: itemOnClick })}
							{#if item.itemType === 'header'}
								<h3 class="text-muted-foreground px-4 pt-2.5 pb-1 text-xs font-semibold uppercase">
									{item.data as string}
								</h3>
							{:else if item.itemType === 'item'}
								{@const snippetItem = item.data as Snippet}
								<button class="w-full text-left" onclick={itemOnClick}>
									<ListItemBase
										icon="snippets-16"
										title={snippetItem.name}
										subtitle={snippetItem.keyword}
										{isSelected}
									/>
								</button>
							{/if}
						{/snippet}
					</BaseList>
				{/if}
			</div>
			<div class="flex flex-col overflow-y-hidden">
				{#if selectedItem}
					<div class="relative flex-grow overflow-y-auto p-4">
						<div class="font-mono text-sm whitespace-pre-wrap">{selectedItem.content}</div>
					</div>

					<InfoList
						title="Information"
						items={[
							{ label: 'Name', value: selectedItem.name },
							{ label: 'Content type', value: 'Text' },
							{ label: 'Times used', value: selectedItem.timesUsed },
							{ label: 'Last used', value: formatDateTime(selectedItem.lastUsedAt) }
						]}
					/>
				{/if}
			</div>
		</div>
	{/snippet}

	{#snippet footer()}
		{#if selectedItem}
			<ActionBar {actions} icon={snippetIcon} title="Search Snippets" />
		{/if}
	{/snippet}
</MainLayout>
