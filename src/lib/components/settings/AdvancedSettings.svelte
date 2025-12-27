<script lang="ts">
	import { settingsStore } from '$lib/settings.svelte';
	import SettingSection from './SettingSection.svelte';
	import SettingItem from './SettingItem.svelte';
	import * as Select from '$lib/components/ui/select';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { Input } from '$lib/components/ui/input';
	import { invoke } from '@tauri-apps/api/core';

	const { settings } = $derived(settingsStore);

	function handleLogLevelChange(value: string | undefined) {
		if (
			value &&
			(value === 'error' ||
				value === 'warn' ||
				value === 'info' ||
				value === 'debug' ||
				value === 'trace')
		) {
			settingsStore.updateSetting('debugLogLevel', value);
		}
	}

	function handleMaxExtensionsChange(e: Event) {
		const value = parseInt((e.target as HTMLInputElement).value);
		if (!isNaN(value) && value > 0) {
			settingsStore.updateSetting('maxConcurrentExtensions', value);
		}
	}

	function handleCacheSizeChange(e: Event) {
		const value = parseInt((e.target as HTMLInputElement).value);
		if (!isNaN(value) && value > 0) {
			settingsStore.updateSetting('cacheSizeMb', value);
		}
	}

	function handleThrottleChange(e: Event) {
		const value = parseInt((e.target as HTMLInputElement).value);
		if (!isNaN(value) && value >= 0) {
			settingsStore.updateSetting('indexingThrottleMs', value);
		}
	}

	function handleRetentionChange(e: Event) {
		const value = parseInt((e.target as HTMLInputElement).value);
		if (!isNaN(value) && value > 0) {
			settingsStore.updateSetting('clipboardHistoryRetentionDays', value);
		}
	}

	async function handleAutoStartChange(checked: boolean) {
		try {
			await invoke('set_auto_start_enabled', { enabled: checked });
			settingsStore.updateSetting('autoStartOnLogin', checked);
		} catch (error) {
			console.error('Failed to set auto-start:', error);
			// Revert the setting on error
			settingsStore.updateSetting('autoStartOnLogin', !checked);
		}
	}
</script>

<div class="h-full overflow-y-auto px-6 py-4">
	<h2 class="mb-6 text-lg font-semibold">Advanced Settings</h2>

	<SettingSection
		title="Developer Options"
		description="Settings for developers and advanced users"
	>
		{#snippet children()}
			<SettingItem label="Developer Mode" description="Enable advanced debugging features">
				{#snippet control()}
					<Checkbox
						checked={settings.developerMode}
						onCheckedChange={(checked) =>
							settingsStore.updateSetting('developerMode', checked === true)}
					/>
				{/snippet}
			</SettingItem>

			<SettingItem
				label="Show Extension Console"
				description="Display console output from extensions"
			>
				{#snippet control()}
					<Checkbox
						checked={settings.showExtensionConsole}
						onCheckedChange={(checked) =>
							settingsStore.updateSetting('showExtensionConsole', checked === true)}
					/>
				{/snippet}
			</SettingItem>

			<SettingItem label="Debug Log Level" description="Control verbosity of logging">
				{#snippet control()}
					<Select.Root
						value={settings.debugLogLevel}
						onValueChange={handleLogLevelChange}
						type="single"
					>
						<Select.Trigger class="w-40">
							{settings.debugLogLevel.charAt(0).toUpperCase() + settings.debugLogLevel.slice(1)}
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="error">Error</Select.Item>
							<Select.Item value="warn">Warning</Select.Item>
							<Select.Item value="info">Info</Select.Item>
							<Select.Item value="debug">Debug</Select.Item>
							<Select.Item value="trace">Trace</Select.Item>
						</Select.Content>
					</Select.Root>
				{/snippet}
			</SettingItem>
		{/snippet}
	</SettingSection>

	<SettingSection title="Performance" description="Configure performance and resource usage">
		{#snippet children()}
			<SettingItem
				label="Max Concurrent Extensions"
				description="Maximum number of extensions that can run simultaneously"
			>
				{#snippet control()}
					<Input
						type="number"
						value={settings.maxConcurrentExtensions}
						onchange={handleMaxExtensionsChange}
						class="w-24"
						min="1"
						max="20"
					/>
				{/snippet}
			</SettingItem>

			<SettingItem label="Cache Size (MB)" description="Maximum cache size in megabytes">
				{#snippet control()}
					<Input
						type="number"
						value={settings.cacheSizeMb}
						onchange={handleCacheSizeChange}
						class="w-24"
						min="10"
						max="1000"
					/>
				{/snippet}
			</SettingItem>

			<SettingItem
				label="Indexing Throttle (ms)"
				description="Delay between file indexing operations"
			>
				{#snippet control()}
					<Input
						type="number"
						value={settings.indexingThrottleMs}
						onchange={handleThrottleChange}
						class="w-24"
						min="0"
						max="5000"
					/>
				{/snippet}
			</SettingItem>
		{/snippet}
	</SettingSection>

	<SettingSection
		title="System Integration"
		description="Configure how Flareup integrates with your system"
	>
		{#snippet children()}
			<SettingItem
				label="Auto-Start on Login"
				description="Launch Flareup automatically when you log in"
			>
				{#snippet control()}
					<Checkbox
						checked={settings.autoStartOnLogin}
						onCheckedChange={(checked) => handleAutoStartChange(checked === true)}
					/>
				{/snippet}
			</SettingItem>

			<SettingItem
				label="Clipboard History Retention"
				description="Number of days to keep clipboard history"
			>
				{#snippet control()}
					<Input
						type="number"
						value={settings.clipboardHistoryRetentionDays}
						onchange={handleRetentionChange}
						class="w-24"
						min="1"
						max="365"
					/>
				{/snippet}
			</SettingItem>
		{/snippet}
	</SettingSection>
</div>
