<script lang="ts">
	import { sidecarService } from '$lib/sidecar.svelte';
	import { Button } from '$lib/components/ui/button';
	import { onMount } from 'svelte';

	type Props = {
		onClose: () => void;
	};

	let { onClose }: Props = $props();

	let scrollAreaRef: HTMLDivElement | null = $state(null);
	let shouldAutoScroll = $state(true);

	const { logs } = $derived(sidecarService);

	// Auto-scroll to bottom when new logs arrive
	$effect(() => {
		if (shouldAutoScroll && scrollAreaRef) {
			scrollAreaRef.scrollTop = scrollAreaRef.scrollHeight;
		}
		// React to logs changes
		logs;
	});

	function handleClearLogs() {
		sidecarService.clearLogs();
	}

	async function copyLogsToClipboard() {
		const logText = logs.join('\n');
		await navigator.clipboard.writeText(logText);
	}

	function handleScroll(event: Event) {
		const target = event.target as HTMLElement;
		const isAtBottom = target.scrollHeight - target.scrollTop - target.clientHeight < 50;
		shouldAutoScroll = isAtBottom;
	}

	onMount(() => {
		// Scroll to bottom on mount
		if (scrollAreaRef) {
			scrollAreaRef.scrollTop = scrollAreaRef.scrollHeight;
		}
	});

	function getLogLevel(message: string): 'error' | 'warn' | 'info' {
		const upperMsg = message.toUpperCase();
		if (upperMsg.includes('ERROR') || upperMsg.includes('FAILED')) return 'error';
		if (upperMsg.includes('WARN')) return 'warn';
		return 'info';
	}

	function getLogStyle(level: 'error' | 'warn' | 'info'): string {
		switch (level) {
			case 'error':
				return 'text-red-400 bg-red-950/20';
			case 'warn':
				return 'text-yellow-400 bg-yellow-950/20';
			default:
				return 'text-foreground';
		}
	}
</script>

<!-- Bottom docked panel -->
<div
	class="bg-background fixed right-0 bottom-0 left-0 z-40 flex h-[35vh] flex-col border-t shadow-2xl"
>
	<!-- Header -->
	<div class="flex items-center justify-between border-b px-4 py-2">
		<div class="flex items-center gap-3">
			<h2 class="text-sm font-semibold">Debug Logs</h2>
			<span class="text-muted-foreground text-xs">({logs.length} entries)</span>
		</div>
		<div class="flex items-center gap-2">
			<Button
				variant="outline"
				size="sm"
				onclick={copyLogsToClipboard}
				disabled={logs.length === 0}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="14"
					height="14"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="mr-1.5"
				>
					<rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
					<path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
				</svg>
				Copy
			</Button>
			<Button variant="outline" size="sm" onclick={handleClearLogs} disabled={logs.length === 0}>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="14"
					height="14"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="mr-1.5"
				>
					<path d="M3 6h18" />
					<path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
					<path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
				</svg>
				Clear
			</Button>
			<Button variant="ghost" size="sm" onclick={onClose}>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="14"
					height="14"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="mr-1.5"
				>
					<path d="M18 6 6 18" />
					<path d="m6 6 12 12" />
				</svg>
				Close
			</Button>
		</div>
	</div>

	<!-- Logs Content -->
	<div class="flex-1 overflow-y-auto p-3" bind:this={scrollAreaRef} onscroll={handleScroll}>
		<div class="space-y-0.5 font-mono text-xs">
			{#if logs.length === 0}
				<div class="text-muted-foreground flex h-full items-center justify-center">
					No logs yet. Logs will appear here when extensions run.
				</div>
			{:else}
				{#each logs as log, i (i)}
					{@const level = getLogLevel(log)}
					{@const style = getLogStyle(level)}
					<div class="rounded px-2 py-1 {style}">
						<span class="text-muted-foreground mr-2 text-[10px]">
							{new Date().toLocaleTimeString()}
						</span>
						<span class="break-all">{log}</span>
					</div>
				{/each}
			{/if}
		</div>
	</div>

	<!-- Footer -->
	<div
		class="text-muted-foreground flex items-center justify-between border-t px-4 py-1.5 text-[10px]"
	>
		<span>Press Cmd/Ctrl + Shift + L to toggle</span>
		<span>Auto-scroll: {shouldAutoScroll ? 'On' : 'Off'}</span>
	</div>
</div>
