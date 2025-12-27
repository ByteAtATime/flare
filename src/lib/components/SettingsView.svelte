<script lang="ts">
	import type { PluginInfo } from '@flare/protocol';
	import * as Tabs from '$lib/components/ui/tabs';
	import AiSettingsView from './AiSettingsView.svelte';
	import HotkeysSettings from './HotkeysSettings.svelte';
	import ExtensionsSettings from './settings/ExtensionsSettings.svelte';
	import GeneralSettings from './settings/GeneralSettings.svelte';
	import AppearanceSettings from './settings/AppearanceSettings.svelte';
	import AdvancedSettings from './settings/AdvancedSettings.svelte';
	import AboutSettings from './settings/AboutSettings.svelte';
	import { settingsStore } from '$lib/settings.svelte';
	import { onMount } from 'svelte';

	type Props = {
		plugins: PluginInfo[];
		onBack: () => void;
		onSavePreferences: (pluginName: string, values: Record<string, unknown>) => void;
		onGetPreferences: (pluginName: string) => void;
		currentPreferences: Record<string, unknown>;
	};

	let { plugins, onBack, onSavePreferences, onGetPreferences, currentPreferences }: Props =
		$props();

	let activeTab = $state('general');

	function applyTheme(theme: string) {
		const root = document.documentElement;

		// Remove all theme classes first
		root.classList.remove(
			'dark',
			'tokyo-night',
			'dracula',
			'nord',
			'catppuccin',
			'gruvbox',
			'one-dark'
		);

		if (theme === 'system') {
			const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
			if (prefersDark) {
				root.classList.add('dark');
			}
		} else if (theme === 'light') {
			// Light theme is default, no class needed
		} else {
			// Apply specific theme class
			root.classList.add(theme);
		}
	}

	onMount(async () => {
		// Load settings on mount if not already loaded
		if (!settingsStore.isLoaded()) {
			await settingsStore.loadSettings();
		}
		// Apply the current theme
		applyTheme(settingsStore.settings.theme);
	});

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape' && !event.defaultPrevented) {
			event.preventDefault();
			onBack();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="bg-background text-foreground h-screen">
	<Tabs.Root bind:value={activeTab} class="h-full pt-2">
		<Tabs.List class="mx-auto">
			<Tabs.Trigger value="general">General</Tabs.Trigger>
			<Tabs.Trigger value="appearance">Appearance</Tabs.Trigger>
			<Tabs.Trigger value="extensions">Extensions</Tabs.Trigger>
			<Tabs.Trigger value="hotkeys">Hotkeys</Tabs.Trigger>
			<Tabs.Trigger value="ai">AI</Tabs.Trigger>
			<Tabs.Trigger value="advanced">Advanced</Tabs.Trigger>
			<Tabs.Trigger value="about">About</Tabs.Trigger>
		</Tabs.List>

		<Tabs.Content value="general">
			<GeneralSettings />
		</Tabs.Content>

		<Tabs.Content value="appearance">
			<AppearanceSettings />
		</Tabs.Content>

		<Tabs.Content value="extensions" class="flex h-full">
			<ExtensionsSettings
				{plugins}
				{onBack}
				{onSavePreferences}
				{onGetPreferences}
				{currentPreferences}
			/>
		</Tabs.Content>

		<Tabs.Content value="hotkeys" class="h-full">
			<HotkeysSettings {plugins} {onBack} />
		</Tabs.Content>

		<Tabs.Content value="ai">
			<AiSettingsView />
		</Tabs.Content>

		<Tabs.Content value="advanced">
			<AdvancedSettings />
		</Tabs.Content>

		<Tabs.Content value="about">
			<AboutSettings />
		</Tabs.Content>
	</Tabs.Root>
</main>
