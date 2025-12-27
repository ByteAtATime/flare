<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount, tick, untrack } from 'svelte';
	import { Loader2, FolderOpen, Trash2, ExternalLink, LayoutGrid, List } from '@lucide/svelte';
	import ListItemBase from './nodes/shared/ListItemBase.svelte';
	import * as Select from './ui/select';
	import ActionBar from './nodes/shared/ActionBar.svelte';
	import BaseList from './BaseList.svelte';
	import HeaderInput from './HeaderInput.svelte';
	import MainLayout from './layout/MainLayout.svelte';
	import Header from './layout/Header.svelte';
	import InfoList from './InfoList.svelte';
	import type { ActionDefinition } from './nodes/shared/actions';

	type Props = {
		onBack: () => void;
	};

	type DownloadItem = {
		id: number;
		path: string;
		name: string;
		fileType: string;
		extension: string | null;
		sizeBytes: number;
		createdAt: string;
		accessedAt: string | null;
		isComplete: boolean;
	};

	let { onBack }: Props = $props();

	let allItems = $state<DownloadItem[]>([]);
	let selectedIndex = $state(0);
	let searchText = $state('');
	let filter = $state('all');
	let sortBy = $state('date');
	let viewMode = $state<'list' | 'grid'>('list');
	let listContainerEl = $state<HTMLElement | null>(null);
	let isInitialMount = $state(true);

	let currentPage = $state(0);
	let hasMore = $state(true);
	let isFetching = $state(false);

	const PAGE_SIZE = 50;

	// Extension to icon mapping
	const getIconForExtension = (ext: string | null): string => {
		if (!ext) return 'blank-document-16';
		const extension = ext.toLowerCase();

		// Images
		if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp', 'ico'].includes(extension)) {
			return 'image-16';
		}
		// Videos
		if (['mp4', 'mov', 'avi', 'mkv', 'webm', 'flv', 'wmv'].includes(extension)) {
			return 'video-16';
		}
		// Audio
		if (['mp3', 'wav', 'flac', 'm4a', 'ogg', 'aac'].includes(extension)) {
			return 'music-16';
		}
		// Documents
		if (['pdf'].includes(extension)) {
			return 'document-16';
		}
		if (['doc', 'docx', 'txt', 'md', 'rtf', 'odt'].includes(extension)) {
			return 'document-16';
		}
		// Spreadsheets
		if (['xls', 'xlsx', 'csv'].includes(extension)) {
			return 'spreadsheet-16';
		}
		// Archives
		if (['zip', 'tar', 'gz', '7z', 'rar', 'bz2', 'xz'].includes(extension)) {
			return 'folder-zip-16';
		}
		// Executables
		if (['exe', 'app', 'dmg', 'deb', 'rpm', 'appimage'].includes(extension)) {
			return 'app-window-16';
		}

		return 'blank-document-16';
	};

	const formatFileSize = (bytes: number): string => {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
	};

	const formatDateTime = (dateString: string): string => {
		const date = new Date(dateString);
		const now = new Date();
		const isToday = date.toDateString() === now.toDateString();

		if (isToday) {
			return `Today at ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
		}

		const yesterday = new Date(now);
		yesterday.setDate(yesterday.getDate() - 1);
		if (date.toDateString() === yesterday.toDateString()) {
			return `Yesterday at ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
		}

		return date.toLocaleDateString([], {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	};

	const loadMoreItems = async () => {
		if (isFetching || !hasMore) return;
		isFetching = true;
		try {
			const newItems = await invoke<DownloadItem[]>('downloads_get_items', {
				filter,
				limit: PAGE_SIZE,
				offset: currentPage * PAGE_SIZE,
				searchTerm: searchText || null
			});
			if (newItems.length < PAGE_SIZE) hasMore = false;
			allItems = currentPage === 0 ? newItems : [...allItems, ...newItems];
			currentPage += 1;
		} catch (e) {
			console.error('Failed to fetch downloads:', e);
		} finally {
			isFetching = false;
		}
	};

	const resetAndFetch = () => {
		allItems = [];
		currentPage = 0;
		hasMore = true;
		if (isFetching) return;
		selectedIndex = 0;
		tick().then(loadMoreItems);
	};

	const handleOpen = async (item: DownloadItem) => {
		try {
			await invoke('downloads_open_file', { path: item.path });
		} catch (e) {
			console.error('Failed to open file:', e);
		}
	};

	const handleShowInFolder = async (item: DownloadItem) => {
		try {
			await invoke('downloads_show_in_folder', { path: item.path });
		} catch (e) {
			console.error('Failed to show in folder:', e);
		}
	};

	const handleDeleteFromHistory = async (item: DownloadItem) => {
		try {
			await invoke('downloads_delete_item', { id: item.id });
			resetAndFetch();
		} catch (e) {
			console.error('Failed to delete from history:', e);
		}
	};

	const handleClearHistory = async () => {
		try {
			await invoke('downloads_clear_history');
			resetAndFetch();
		} catch (e) {
			console.error('Failed to clear history:', e);
		}
	};

	onMount(() => {
		const container = listContainerEl;
		if (!container) return;
		const onScroll = () => {
			if (
				container.scrollHeight > container.clientHeight &&
				container.scrollHeight - container.scrollTop - container.clientHeight < 200
			) {
				loadMoreItems();
			}
		};
		container.addEventListener('scroll', onScroll);
		resetAndFetch();
		isInitialMount = false;
		return () => container.removeEventListener('scroll', onScroll);
	});

	$effect(() => {
		[searchText, filter, sortBy];
		if (isInitialMount) return;

		untrack(() => {
			resetAndFetch();
		});
	});

	const selectedItem = $derived(allItems[selectedIndex] ?? null);

	const actions: ActionDefinition[] = $derived(
		selectedItem
			? [
					{
						title: 'Open',
						handler: () => handleOpen(selectedItem)
					},
					{
						title: 'Show in Folder',
						shortcut: { key: 'O', modifiers: ['cmd', 'shift'] },
						handler: () => handleShowInFolder(selectedItem)
					},
					{
						title: 'Remove from History',
						shortcut: { key: 'x', modifiers: ['ctrl'] },
						handler: () => handleDeleteFromHistory(selectedItem)
					}
				]
			: []
	);
</script>

<MainLayout>
	{#snippet header()}
		<Header showBackButton={true} onPopView={onBack}>
			<HeaderInput
				placeholder="Search downloads..."
				bind:value={searchText}
				autofocus
				class="!pl-2.5"
			/>
			{#snippet actions()}
				<Select.Root bind:value={sortBy} type="single">
					<Select.Trigger class="w-32">
						{sortBy === 'date' ? 'Date Added' : sortBy === 'name' ? 'Name' : 'Modified'}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="date">Date Added</Select.Item>
						<Select.Item value="name">Name</Select.Item>
						<Select.Item value="modified">Modified</Select.Item>
					</Select.Content>
				</Select.Root>
				<Select.Root bind:value={filter} type="single">
					<Select.Trigger class="w-32">
						{filter === 'all' ? 'All Files' : filter.charAt(0).toUpperCase() + filter.slice(1)}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="all">All Files</Select.Item>
						<Select.Item value="images">Images</Select.Item>
						<Select.Item value="videos">Videos</Select.Item>
						<Select.Item value="audio">Audio</Select.Item>
						<Select.Item value="documents">Documents</Select.Item>
						<Select.Item value="archives">Archives</Select.Item>
					</Select.Content>
				</Select.Root>
				<button
					onclick={() => (viewMode = viewMode === 'list' ? 'grid' : 'list')}
					class="hover:bg-accent rounded-md p-2 transition-colors"
					title={viewMode === 'list' ? 'Switch to Grid View' : 'Switch to List View'}
				>
					{#if viewMode === 'list'}
						<LayoutGrid class="size-4" />
					{:else}
						<List class="size-4" />
					{/if}
				</button>
			{/snippet}
		</Header>
	{/snippet}
	{#snippet content()}
		<div class="grid grow grid-cols-[minmax(0,_1.5fr)_minmax(0,_2.5fr)] overflow-y-hidden">
			<div class="flex-grow overflow-y-auto border-r" bind:this={listContainerEl}>
				{#if allItems.length === 0 && !isFetching}
					<div
						class="text-muted-foreground flex h-full items-center justify-center p-8 text-center"
					>
						<p>No downloads found</p>
					</div>
				{:else if viewMode === 'list'}
					<BaseList items={allItems} bind:selectedIndex onenter={(item) => handleOpen(item)}>
						{#snippet itemSnippet({ item, isSelected, onclick: itemOnClick })}
							<button class="w-full" onclick={itemOnClick}>
								<ListItemBase
									icon={getIconForExtension(item.extension)}
									title={item.name}
									subtitle={formatFileSize(item.sizeBytes)}
									{isSelected}
								/>
							</button>
						{/snippet}
					</BaseList>
				{:else}
					<!-- Grid View -->
					<div class="grid grid-cols-3 gap-4 p-4">
						{#each allItems as item, index}
							<button
								onclick={() => (selectedIndex = index)}
								ondblclick={() => handleOpen(item)}
								class="hover:bg-accent/50 flex flex-col items-center gap-2 rounded-lg border p-4 transition-colors {selectedIndex ===
								index
									? 'bg-accent border-primary'
									: 'border-transparent'}"
							>
								{#if item.extension && ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'].includes(item.extension.toLowerCase())}
									<img
										src={`file://${item.path}`}
										alt={item.name}
										class="h-24 w-full rounded object-cover"
										onerror={(e) => {
											(e.target as HTMLImageElement).style.display = 'none';
										}}
									/>
								{:else}
									<div class="flex h-24 w-full items-center justify-center rounded bg-black/10">
										<span class="text-muted-foreground text-2xl font-semibold uppercase">
											{item.extension || '?'}
										</span>
									</div>
								{/if}
								<p class="w-full truncate text-center text-sm">{item.name}</p>
								<p class="text-muted-foreground text-xs">{formatFileSize(item.sizeBytes)}</p>
							</button>
						{/each}
					</div>
				{/if}
				{#if isFetching && allItems.length > 0}
					<div class="text-muted-foreground flex h-10 items-center justify-center">
						<Loader2 class="size-4 animate-spin" />
					</div>
				{/if}
			</div>
			<div class="flex flex-col overflow-y-hidden">
				{#if selectedItem}
					<div class="relative flex-grow overflow-y-auto p-4">
						<div class="flex flex-col items-center justify-center gap-4 py-8">
							<!-- File icon preview -->
							<div class="flex size-24 items-center justify-center rounded-lg bg-black/10">
								{#if selectedItem.extension && ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'].includes(selectedItem.extension.toLowerCase())}
									<!-- For images, we could show a thumbnail, but for now show icon -->
									<img
										src={`file://${selectedItem.path}`}
										alt={selectedItem.name}
										class="max-h-full max-w-full rounded object-contain"
										onerror={(e) => {
											// If image fails to load, hide it
											(e.target as HTMLImageElement).style.display = 'none';
										}}
									/>
								{:else}
									<span class="text-muted-foreground text-3xl font-semibold uppercase">
										{selectedItem.extension || '?'}
									</span>
								{/if}
							</div>
							<p class="max-w-full truncate text-center font-medium">{selectedItem.name}</p>
						</div>
					</div>

					<InfoList
						title="File Info"
						items={[
							{ label: 'Size', value: formatFileSize(selectedItem.sizeBytes) },
							{ label: 'Type', value: selectedItem.extension?.toUpperCase() || 'Unknown' },
							{ label: 'Downloaded', value: formatDateTime(selectedItem.createdAt) },
							...(selectedItem.accessedAt
								? [{ label: 'Last Opened', value: formatDateTime(selectedItem.accessedAt) }]
								: [])
						]}
					/>
				{:else}
					<div class="text-muted-foreground flex h-full items-center justify-center">
						<p>Select a download to view details</p>
					</div>
				{/if}
			</div>
		</div>
	{/snippet}

	{#snippet footer()}
		{#if selectedItem}
			<ActionBar {actions} title="Downloads" />
		{/if}
	{/snippet}
</MainLayout>
