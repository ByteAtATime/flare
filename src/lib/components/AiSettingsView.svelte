<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Switch } from './ui/switch';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { APP_VERSION } from '$lib/version';
	import PasswordInput from './PasswordInput.svelte';
	import * as Select from './ui/select';
	import { uiStore } from '$lib/ui.svelte';
	import AiPresetsSettings from './AiPresetsSettings.svelte';
	import { Separator } from './ui/separator';

	type AiSettings = {
		enabled: boolean;
		provider: 'openRouter' | 'ollama';
		baseUrl?: string;
		temperature: number;
		modelAssociations: Record<string, string>;
	};

	let aiEnabled = $state(false);
	let aiProvider = $state<'openRouter' | 'ollama'>('openRouter');
	let baseUrl = $state('');
	let temperature = $state(0.7);
	let apiKey = $state('');
	let modelAssociations = $state<Record<string, string>>({});
	let isApiKeySet = $state(false);
	let ollamaModels = $state<string[]>([]);
	let isLoadingModels = $state(false);
	let isSaving = $state(false);

	async function fetchOllamaModels() {
		if (aiProvider !== 'ollama') return;
		isLoadingModels = true;
		try {
			const models = await invoke<string[]>('get_ollama_models', {
				baseUrl: baseUrl || 'http://localhost:11434/v1'
			});
			ollamaModels = models;
		} catch (error) {
			console.error('Failed to fetch Ollama models:', error);
			uiStore.toasts.set(Date.now(), {
				id: Date.now(),
				title: 'Failed to fetch Ollama models',
				message: String(error),
				style: 'FAILURE'
			});
		} finally {
			isLoadingModels = false;
		}
	}

	async function loadSettings() {
		try {
			isApiKeySet = await invoke('is_ai_api_key_set');
			const settings = await invoke<AiSettings>('get_ai_settings');
			aiEnabled = settings.enabled;
			aiProvider = settings.provider || 'openRouter';
			baseUrl = settings.baseUrl || '';
			temperature = settings.temperature ?? 0.7;
			modelAssociations = settings.modelAssociations ?? {};
			if (aiProvider === 'ollama') {
				fetchOllamaModels();
			}
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

	$effect(() => {
		if (aiProvider === 'ollama') {
			fetchOllamaModels();
		}
	});

	async function saveSettings() {
		isSaving = true;
		try {
			if (apiKey) {
				await invoke('set_ai_api_key', { key: apiKey });
				apiKey = '';
			}

			const settingsToSave: AiSettings = {
				enabled: aiEnabled,
				provider: aiProvider,
				baseUrl: baseUrl,
				temperature: temperature,
				modelAssociations: modelAssociations
			};

			await invoke('set_ai_settings', { settings: settingsToSave });

			uiStore.toasts.set(Date.now(), {
				id: Date.now(),
				title: 'Settings saved successfully',
				message: `AI provider: ${aiProvider === 'ollama' ? 'Ollama (Local)' : 'OpenRouter'}`,
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
		} finally {
			isSaving = false;
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
		<h3 class="text-lg font-medium">AI Provider</h3>
		<Select.Root
			type="single"
			value={aiProvider}
			onValueChange={(v) => {
				aiProvider = v as 'openRouter' | 'ollama';
			}}
		>
			<Select.Trigger class="w-full">
				{aiProvider === 'openRouter' ? 'OpenRouter' : 'Ollama (Local)'}
			</Select.Trigger>
			<Select.Content>
				<Select.Item value="openRouter">OpenRouter</Select.Item>
				<Select.Item value="ollama">Ollama (Local)</Select.Item>
			</Select.Content>
		</Select.Root>
	</div>

	{#if aiProvider === 'openRouter'}
		<div class="space-y-2">
			<h3 class="text-lg font-medium">OpenRouter API Key</h3>
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
	{:else}
		<div class="space-y-2">
			<h3 class="text-lg font-medium">Ollama Base URL</h3>
			<p class="text-muted-foreground text-sm">
				The endpoint for your local Ollama instance (default: http://localhost:11434/v1).
			</p>
			<Input bind:value={baseUrl} placeholder="http://localhost:11434/v1" />
		</div>
	{/if}

	<div class="space-y-2">
		<h3 class="text-lg font-medium">Temperature</h3>
		<p class="text-muted-foreground text-sm">
			Controls randomness: 0 is focused and deterministic, 1 is creative and varied. Default: 0.7
		</p>
		<div class="flex items-center gap-4">
			<input type="range" min="0" max="1" step="0.1" bind:value={temperature} class="flex-1" />
			<span class="w-12 text-right font-mono text-sm">{temperature.toFixed(1)}</span>
		</div>
	</div>

	<div class="space-y-2">
		<h3 class="text-lg font-medium">
			{aiProvider === 'ollama' ? 'Default Model' : 'Model Associations'}
		</h3>
		<p class="text-muted-foreground text-sm">
			{#if aiProvider === 'ollama'}
				Select the default Ollama model to use for AI chat.
			{:else}
				Associate internal model identifiers with specific models available through OpenRouter.
			{/if}
		</p>

		{#if aiProvider === 'ollama'}
			<Select.Root
				type="single"
				value={modelAssociations['default'] || ''}
				onValueChange={(v) => (modelAssociations['default'] = v)}
			>
				<Select.Trigger class="w-full">
					{ollamaModels.includes(modelAssociations['default'] || '')
						? modelAssociations['default']
						: 'Select a local model'}
				</Select.Trigger>
				<Select.Content>
					{#if isLoadingModels}
						<Select.Item value="" disabled>Loading models...</Select.Item>
					{:else if ollamaModels.length === 0}
						<Select.Item value="" disabled
							>No models found. Is Ollama running at {baseUrl ||
								'http://localhost:11434/v1'}?</Select.Item
						>
					{:else}
						{#each ollamaModels as model}
							<Select.Item value={model}>{model}</Select.Item>
						{/each}
					{/if}
				</Select.Content>
			</Select.Root>
		{:else}
			<div class="grid grid-cols-[auto_1fr] items-center gap-4">
				{#each Object.entries(modelAssociations) as [raycastModel, selectedModel] (raycastModel)}
					<span class="text-sm font-medium">{raycastModel}</span>
					<Input
						value={selectedModel}
						onchange={(e) => {
							modelAssociations[raycastModel] = (e.target as HTMLInputElement)?.value;
						}}
						class="w-full"
					/>
				{/each}
			</div>
		{/if}
	</div>
	<div class="flex justify-end">
		<Button onclick={saveSettings} disabled={isSaving}>
			{isSaving ? 'Saving...' : 'Save AI Settings'}
		</Button>
	</div>

	<Separator />

	<AiPresetsSettings />

	<div class="text-muted-foreground mt-4 text-center text-xs">Flareup v{APP_VERSION}</div>
</div>
