<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import type { PluginInfo } from '@flare/protocol';
	import { onMount } from 'svelte';

	interface HotkeyConfig {
		commandId: string;
		hotkey: string;
		modifiers: number;
		key: string;
	}

	export let plugins: PluginInfo[];
	export let onBack: () => void;

	let hotkeys: HotkeyConfig[] = [];
	let recordingFor: string | null = null;
	let conflictWarning: string | null = null;
	let pressedModifiers = 0;
	let pressedKey = '';

	onMount(async () => {
		await loadHotkeys();
	});

	async function loadHotkeys() {
		try {
			hotkeys = await invoke<HotkeyConfig[]>('get_hotkey_config');
		} catch (e) {
			console.error('Failed to load hotkeys:', e);
		}
	}

	function getHotkey(commandId: string): HotkeyConfig | undefined {
		return hotkeys.find((h) => h.commandId === commandId);
	}

	function startRecording(commandId: string) {
		recordingFor = commandId;
		conflictWarning = null;
		pressedModifiers = 0;
		pressedKey = '';
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (!recordingFor) return;

		event.preventDefault();
		event.stopPropagation();

		// Calculate modifiers bitmask
		pressedModifiers = 0;
		if (event.ctrlKey) pressedModifiers |= 1;
		if (event.altKey) pressedModifiers |= 2;
		if (event.shiftKey) pressedModifiers |= 4;
		if (event.metaKey) pressedModifiers |= 8;

		// Get key code
		const key = event.code;

		// Ignore modifier-only presses
		if (
			key === 'Control' ||
			key === 'Alt' ||
			key === 'Shift' ||
			key === 'Meta' ||
			key === 'ControlLeft' ||
			key === 'ControlRight' ||
			key === 'AltLeft' ||
			key === 'AltRight' ||
			key === 'ShiftLeft' ||
			key === 'ShiftRight' ||
			key === 'MetaLeft' ||
			key === 'MetaRight'
		) {
			return;
		}

		// Require at least one modifier
		if (pressedModifiers === 0) {
			conflictWarning = 'Hotkeys must include at least one modifier (Ctrl, Alt, Shift, or Super)';
			return;
		}

		pressedKey = key;
	}

	async function saveHotkey() {
		if (!recordingFor || !pressedKey) return;

		try {
			// Check for conflicts
			const conflict = await invoke<string | null>('check_hotkey_conflict', {
				modifiers: pressedModifiers,
				key: pressedKey
			});

			if (conflict && conflict !== recordingFor) {
				const conflictPlugin = plugins.find((p) => p.pluginPath === conflict);
				conflictWarning = `Already assigned to: ${conflictPlugin?.title || conflict}`;
				return;
			}

			// Save the hotkey
			await invoke('set_command_hotkey', {
				commandId: recordingFor,
				modifiers: pressedModifiers,
				key: pressedKey
			});

			// Reload hotkeys
			await loadHotkeys();

			// Clear recording state
			recordingFor = null;
			conflictWarning = null;
			pressedModifiers = 0;
			pressedKey = '';
		} catch (e) {
			console.error('Failed to save hotkey:', e);
			conflictWarning = `Error: ${e}`;
		}
	}

	function cancelRecording() {
		recordingFor = null;
		conflictWarning = null;
		pressedModifiers = 0;
		pressedKey = '';
	}

	async function removeHotkey(commandId: string) {
		try {
			await invoke('remove_command_hotkey', { commandId });
			await loadHotkeys();
		} catch (e) {
			console.error('Failed to remove hotkey:', e);
		}
	}

	async function resetToDefaults() {
		if (confirm('Reset all hotkeys to defaults? This will remove your custom configurations.')) {
			try {
				await invoke('reset_hotkeys_to_defaults');
				await loadHotkeys();
			} catch (e) {
				console.error('Failed to reset hotkeys:', e);
			}
		}
	}

	function formatModifiers(mods: number): string[] {
		const parts: string[] = [];
		if (mods & 8) parts.push('Super');
		if (mods & 1) parts.push('Ctrl');
		if (mods & 2) parts.push('Alt');
		if (mods & 4) parts.push('Shift');
		return parts;
	}

	function formatKey(key: string): string {
		if (key.startsWith('Key')) return key.slice(3); // "KeyV" -> "V"
		if (key.startsWith('Digit')) return key.slice(5); // "Digit5" -> "5"
		if (key === 'ArrowLeft') return '←';
		if (key === 'ArrowRight') return '→';
		if (key === 'ArrowUp') return '↑';
		if (key === 'ArrowDown') return '↓';
		if (key === 'Space') return 'Space';
		if (key === 'Enter') return 'Enter';
		if (key === 'Minus') return '-';
		if (key === 'Equal') return '=';
		return key;
	}

	function formatCurrentRecording(): string {
		if (!pressedKey) return 'Press a key combination...';
		const mods = formatModifiers(pressedModifiers);
		const key = formatKey(pressedKey);
		return [...mods, key].join('+');
	}
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="hotkeys-settings">
	<div class="mb-8 flex items-center justify-between gap-5">
		<button
			class="bg-secondary hover:bg-secondary/80 border-border text-foreground rounded-md border px-4 py-2 text-sm transition-colors"
			on:click={onBack}
		>
			← Back
		</button>
		<h1 class="text-foreground flex-1 text-2xl font-semibold">
			Hotkey Configuration
			<span class="text-muted-foreground ml-2 text-xs font-normal">v1.0.1</span>
		</h1>
		<button
			class="bg-secondary hover:bg-secondary/80 border-border text-foreground rounded-md border px-4 py-2 text-sm transition-colors"
			on:click={resetToDefaults}
		>
			Reset to Defaults
		</button>
	</div>

	<div class="hotkey-list">
		{#each plugins as plugin}
			{@const hotkey = getHotkey(plugin.pluginPath)}
			<div
				class="bg-background border-border hover:border-muted-foreground flex items-center justify-between rounded-lg border p-4 transition-all hover:shadow-md"
			>
				<div class="min-w-0 flex-1">
					<div class="text-foreground mb-1 text-base font-medium">{plugin.title}</div>
					<div class="text-muted-foreground mb-1 text-sm">{plugin.description}</div>
					<div class="text-muted-foreground font-mono text-xs opacity-70">{plugin.pluginPath}</div>
				</div>

				<div class="ml-4 flex min-w-[200px] justify-end">
					{#if recordingFor === plugin.pluginPath}
						<div
							class="bg-accent border-primary flex min-w-[250px] flex-col gap-2 rounded-md border-2 p-3"
						>
							<div
								class="bg-background border-border flex h-10 items-center justify-center rounded border px-2 font-mono text-sm font-medium"
							>
								{formatCurrentRecording()}
							</div>
							<div class="flex gap-2">
								<button
									class="hover:bg-primary/90 bg-primary flex-1 rounded border border-green-600 px-3 py-1.5 text-sm text-white transition-colors disabled:cursor-not-allowed disabled:opacity-50"
									on:click={saveHotkey}
									disabled={!pressedKey}
								>
									Save
								</button>
								<button
									class="bg-secondary hover:bg-secondary/80 border-border flex-1 rounded border px-3 py-1.5 text-sm transition-colors"
									on:click={cancelRecording}
								>
									Cancel
								</button>
							</div>
						</div>
					{:else if hotkey}
						<div
							class="bg-secondary border-border flex items-center gap-2 rounded-md border px-3 py-2"
						>
							<span class="font-mono text-sm font-medium">{hotkey.hotkey}</span>
							<button
								class="hover:text-destructive text-muted-foreground text-xl leading-none transition-colors"
								on:click={() => removeHotkey(plugin.pluginPath)}
							>
								×
							</button>
						</div>
					{:else}
						<button
							class="hover:bg-primary border-primary hover:text-primary-foreground rounded-md border bg-transparent px-4 py-2 text-sm transition-all"
							on:click={() => startRecording(plugin.pluginPath)}
						>
							Set Hotkey
						</button>
					{/if}
				</div>
			</div>
		{/each}
	</div>

	{#if conflictWarning}
		<div class="warning-box">{conflictWarning}</div>
	{/if}
</div>

<style>
	.hotkeys-settings {
		padding: 20px;
		max-width: 1000px;
		margin: 0 auto;
	}

	.hotkey-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.warning-box {
		margin-top: 20px;
		padding: 12px 16px;
		border-radius: 6px;
		background: var(--warning-bg, #fff3cd);
		border: 1px solid var(--warning, #ffc107);
		color: var(--warning-text, #856404);
		font-size: 14px;
	}
</style>
