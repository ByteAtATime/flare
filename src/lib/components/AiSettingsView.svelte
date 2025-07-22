<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Switch } from './ui/switch';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import PasswordInput from './PasswordInput.svelte';
	import { uiStore } from '$lib/ui.svelte';

	type AiSettings = {
		enabled: boolean;
		modelAssociations: Record<string, string>;
	};

	let aiEnabled = $state(false);
	let apiKey = $state('');
	let modelAssociations = $state<Record<string, string>>({});
	let isApiKeySet = $state(false);

	async function loadSettings() {
		try {
			isApiKeySet = await invoke('is_ai_api_key_set');
			const settings = await invoke<AiSettings>('get_ai_settings');
			aiEnabled = settings.enabled;
			modelAssociations = settings.modelAssociations ?? {};
		} catch (error) {
			console.error('Failed to load AI settings:', error);
			uiStore.toasts.set(Date.now(), {
				id: Date.now(),
				title: 'Failed to load AI settings',
				message: String(error),
				style: 'FAILURE'
			});
		}
	}

	async function saveSettings() {
		try {
			if (apiKey) {
				await invoke('set_ai_api_key', { key: apiKey });
				apiKey = '';
			}

			const settingsToSave: AiSettings = {
				enabled: aiEnabled,
				modelAssociations: modelAssociations
			};

			await invoke('set_ai_settings', { settings: settingsToSave });

			uiStore.toasts.set(Date.now(), {
				id: Date.now(),
				title: 'AI Settings Saved',
				style: 'SUCCESS'
			});

			await loadSettings();
		} catch (error) {
			console.error('Failed to save AI settings:', error);
			uiStore.toasts.set(Date.now(), {
				id: Date.now(),
				title: 'Failed to save AI settings',
				message: String(error),
				style: 'FAILURE'
			});
		}
	}

	async function clearApiKey() {
		await invoke('clear_ai_api_key');
		apiKey = '';
		await loadSettings();
	}

	onMount(loadSettings);
</script>

<div class="mx-auto max-w-screen-md space-y-6 p-6">
	<div class="space-y-2">
		<h3 class="text-lg font-medium">General AI Settings</h3>
		<div class="flex items-center space-x-2">
			<Switch bind:checked={aiEnabled} id="ai-enabled" />
			<label for="ai-enabled" class="text-sm font-medium"> Enable AI Features </label>
		</div>
	</div>

	<div class="space-y-2">
		<h3 class="text-lg font-medium">API Key</h3>
		<p class="text-muted-foreground text-sm">
			Your OpenRouter API key is stored securely in your system's keychain.
		</p>
		<div class="flex items-center gap-2">
			<PasswordInput
				bind:value={apiKey}
				placeholder={isApiKeySet ? '••••••••••••' : 'Enter your OpenRouter API key'}
				class="flex-grow"
			/>
			{#if isApiKeySet}
				<Button variant="destructive" onclick={clearApiKey}>Clear</Button>
			{/if}
		</div>
	</div>

	<div class="space-y-2">
		<h3 class="text-lg font-medium">Model Associations</h3>
		<p class="text-muted-foreground text-sm">
			Associate internal model identifiers with specific models available through OpenRouter.
		</p>
		<div class="grid grid-cols-[auto_1fr] items-center gap-4">
			{#each Object.entries(modelAssociations) as [raycastModel, openRouterModel] (raycastModel)}
				<span class="text-sm font-medium">{raycastModel}</span>
				<Input
					value={openRouterModel}
					onchange={(e) => {
						modelAssociations[raycastModel] = (e.target as HTMLInputElement)?.value;
					}}
					class="w-full"
				/>
			{/each}
		</div>
	</div>
	<div class="flex justify-end">
		<Button onclick={saveSettings}>Save AI Settings</Button>
	</div>
</div>
