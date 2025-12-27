<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { onMount } from 'svelte';
	import { X } from '@lucide/svelte';

	let content = $state('');
	let isSaving = $state(false);
	let saveTimeout: ReturnType<typeof setTimeout>;

	onMount(async () => {
		try {
			content = await invoke('get_floating_note');
		} catch (error) {
			console.error('Failed to load note:', error);
		}
	});

	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement;
		content = target.value;
		isSaving = true;

		clearTimeout(saveTimeout);
		saveTimeout = setTimeout(async () => {
			try {
				await invoke('save_floating_note', { content });
			} catch (error) {
				console.error('Failed to save note:', error);
			} finally {
				isSaving = false;
			}
		}, 500);
	}

	function closeWindow() {
		getCurrentWebviewWindow().hide();
	}

    // Drag region logic needed for frameless window
    function startDrag() {
        getCurrentWebviewWindow().startDragging();
    }
</script>

<div class="flex h-screen w-full flex-col overflow-hidden rounded-lg border bg-[#fff9c4] shadow-xl dark:bg-yellow-900/90 text-black dark:text-white">
	<!-- Drag Handle / Header -->
	<div
		class="flex h-8 shrink-0 items-center justify-between bg-black/5 px-2 cursor-grab active:cursor-grabbing"
		onmousedown={startDrag}
        role="button"
        tabindex="-1"
	>
		<span class="text-xs font-medium opacity-50">Floating Note</span>
		<button
			onclick={closeWindow}
			class="flex size-5 items-center justify-center rounded-full hover:bg-black/10"
		>
			<X class="size-3" />
		</button>
	</div>

	<textarea
		class="flex-1 resize-none bg-transparent p-3 text-sm leading-relaxed outline-none placeholder:text-black/30 dark:placeholder:text-white/30"
		placeholder="Type your notes here..."
		value={content}
		oninput={handleInput}
        spellcheck="false"
	></textarea>

	<div class="flex h-5 shrink-0 items-center justify-end px-2">
		<span class="text-[10px] opacity-40 transition-opacity duration-300" class:opacity-100={isSaving}>
			{isSaving ? 'Saving...' : 'Saved'}
		</span>
	</div>
</div>

<style>
    :global(body) {
        background: transparent;
    }
</style>
