<script lang="ts">
	import { settingsStore } from '$lib/settings.svelte';
	import SettingSection from './SettingSection.svelte';
	import SettingItem from './SettingItem.svelte';
	import * as Select from '$lib/components/ui/select';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { Input } from '$lib/components/ui/input';

	const { settings } = $derived(settingsStore);

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
			// Check system preference
			const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
			if (prefersDark) {
				root.classList.add('dark');
			}
		} else if (theme === 'light') {
			// Light theme is the default, no class needed
		} else {
			// Apply specific theme class (dark, tokyo-night, dracula, etc.)
			root.classList.add(theme);
		}
	}

	function handleThemeChange(value: string | undefined) {
		const validThemes = [
			'light',
			'dark',
			'tokyo-night',
			'dracula',
			'nord',
			'catppuccin',
			'gruvbox',
			'one-dark',
			'system'
		];
		if (value && validThemes.includes(value)) {
			settingsStore.updateSetting('theme', value as any);
			applyTheme(value);
		}
	}

	function formatThemeName(theme: string): string {
		return theme
			.split('-')
			.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
			.join(' ');
	}

	function handleFontSizeChange(value: string | undefined) {
		if (value && (value === 'small' || value === 'medium' || value === 'large')) {
			settingsStore.updateSetting('fontSize', value);
		}
	}

	function handleFuzzySearchChange(value: string | undefined) {
		if (value && (value === 'low' || value === 'medium' || value === 'high')) {
			settingsStore.updateSetting('fuzzySearchSensitivity', value);
		}
	}

	function handleSearchLimitChange(e: Event) {
		const value = parseInt((e.target as HTMLInputElement).value);
		if (!isNaN(value) && value > 0) {
			settingsStore.updateSetting('searchResultsLimit', value);
		}
	}

	function handleWindowWidthChange(e: Event) {
		const value = parseInt((e.target as HTMLInputElement).value);
		if (!isNaN(value) && value > 0) {
			settingsStore.updateSetting('defaultWindowWidth', value);
		}
	}

	function handleWindowHeightChange(e: Event) {
		const value = parseInt((e.target as HTMLInputElement).value);
		if (!isNaN(value) && value > 0) {
			settingsStore.updateSetting('defaultWindowHeight', value);
		}
	}
</script>

<div class="h-full overflow-y-auto px-6 py-4">
	<h2 class="mb-6 text-lg font-semibold">General Settings</h2>

	<SettingSection title="Appearance" description="Customize how Flareup looks">
		{#snippet children()}
			<SettingItem label="Theme">
				{#snippet control()}
					<Select.Root value={settings.theme} onValueChange={handleThemeChange} type="single">
						<Select.Trigger class="w-40">
							{formatThemeName(settings.theme)}
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="system">System</Select.Item>
							<Select.Item value="light">Light</Select.Item>
							<Select.Item value="dark">Dark</Select.Item>
							<Select.Item value="tokyo-night">Tokyo Night</Select.Item>
							<Select.Item value="dracula">Dracula</Select.Item>
							<Select.Item value="nord">Nord</Select.Item>
							<Select.Item value="catppuccin">Catppuccin</Select.Item>
							<Select.Item value="gruvbox">Gruvbox</Select.Item>
							<Select.Item value="one-dark">One Dark</Select.Item>
						</Select.Content>
					</Select.Root>
				{/snippet}
			</SettingItem>

			<SettingItem label="Font Size">
				{#snippet control()}
					<Select.Root value={settings.fontSize} onValueChange={handleFontSizeChange} type="single">
						<Select.Trigger class="w-40">
							{settings.fontSize.charAt(0).toUpperCase() + settings.fontSize.slice(1)}
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="small">Small</Select.Item>
							<Select.Item value="medium">Medium</Select.Item>
							<Select.Item value="large">Large</Select.Item>
						</Select.Content>
					</Select.Root>
				{/snippet}
			</SettingItem>
		{/snippet}
	</SettingSection>

	<SettingSection title="Search" description="Configure search behavior">
		{#snippet children()}
			<SettingItem label="Enable Search History" description="Remember your recent searches">
				{#snippet control()}
					<Checkbox
						checked={settings.enableSearchHistory}
						onCheckedChange={(checked) =>
							settingsStore.updateSetting('enableSearchHistory', checked === true)}
					/>
				{/snippet}
			</SettingItem>

			<SettingItem label="Results Limit" description="Maximum number of results to show">
				{#snippet control()}
					<Input
						type="number"
						value={settings.searchResultsLimit}
						onchange={handleSearchLimitChange}
						class="w-24"
						min="1"
					/>
				{/snippet}
			</SettingItem>

			<SettingItem label="Fuzzy Search Sensitivity">
				{#snippet control()}
					<Select.Root
						value={settings.fuzzySearchSensitivity}
						onValueChange={handleFuzzySearchChange}
						type="single"
					>
						<Select.Trigger class="w-40">
							{settings.fuzzySearchSensitivity.charAt(0).toUpperCase() +
								settings.fuzzySearchSensitivity.slice(1)}
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="low">Low</Select.Item>
							<Select.Item value="medium">Medium</Select.Item>
							<Select.Item value="high">High</Select.Item>
						</Select.Content>
					</Select.Root>
				{/snippet}
			</SettingItem>
		{/snippet}
	</SettingSection>

	<SettingSection title="Window Behavior" description="Control how the window behaves">
		{#snippet children()}
			<SettingItem label="Close on Blur" description="Hide the window when clicking outside">
				{#snippet control()}
					<Checkbox
						checked={settings.closeOnBlur}
						onCheckedChange={(checked) =>
							settingsStore.updateSetting('closeOnBlur', checked === true)}
					/>
				{/snippet}
			</SettingItem>

			<SettingItem
				label="Remember Window Position"
				description="Save window position between sessions"
			>
				{#snippet control()}
					<Checkbox
						checked={settings.rememberWindowPosition}
						onCheckedChange={(checked) =>
							settingsStore.updateSetting('rememberWindowPosition', checked === true)}
					/>
				{/snippet}
			</SettingItem>

			<SettingItem label="Default Width" description="Default window width in pixels">
				{#snippet control()}
					<Input
						type="number"
						value={settings.defaultWindowWidth}
						onchange={handleWindowWidthChange}
						class="w-24"
						min="400"
					/>
				{/snippet}
			</SettingItem>

			<SettingItem label="Default Height" description="Default window height in pixels">
				{#snippet control()}
					<Input
						type="number"
						value={settings.defaultWindowHeight}
						onchange={handleWindowHeightChange}
						class="w-24"
						min="300"
					/>
				{/snippet}
			</SettingItem>
		{/snippet}
	</SettingSection>
</div>
