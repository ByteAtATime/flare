<script lang="ts">
	import type { PluginInfo, Preference } from '@flare/protocol';
	import { Input } from '$lib/components/ui/input';
	import Icon from '$lib/components/Icon.svelte';
	import { Checkbox } from './ui/checkbox';
	import * as Select from './ui/select';
	import BaseList from './BaseList.svelte';
	import { Button } from './ui/button';
	import path from 'path';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { appsStore } from '$lib/apps.svelte';
	import PasswordInput from './PasswordInput.svelte';
	import * as Tabs from '$lib/components/ui/tabs';
	import AiSettingsView from './AiSettingsView.svelte';
	import { viewManager } from '$lib/viewManager.svelte';

	type Props = {
		plugins: PluginInfo[];
		onBack: () => void;
		onSavePreferences: (pluginName: string, values: Record<string, unknown>) => void;
		onGetPreferences: (pluginName: string) => void;
		currentPreferences: Record<string, unknown>;
	};

	type DisplayListItem = {
		id: string;
		itemType: 'item';
		type: 'extension' | 'command';
		data: PluginInfo;
		isLastInGroup: boolean;
	};

	let { plugins, onBack, onSavePreferences, onGetPreferences, currentPreferences }: Props =
		$props();

	const { pluginToSelectInSettings } = $derived(viewManager);

	let selectedIndex = $state(0);
	let preferenceValues = $state<Record<string, unknown>>({});
	let searchText = $state('');
	let activeTab = $state('extensions');
	const { apps } = $derived(appsStore);

	$effect(() => {
		// This effect syncs the local preference values with the prop.
		// It's necessary because the form is a mutable copy of the preferences
		// that needs to be reset when the selected plugin changes.
		// eslint-disable-next-line svelte/prefer-writable-derived
		preferenceValues = { ...currentPreferences };
	});

	const displayItems = $derived.by(() => {
		const items: DisplayListItem[] = [];
		const extensions = new Map<string, PluginInfo[]>();

		for (const plugin of plugins) {
			if (!extensions.has(plugin.pluginName)) {
				extensions.set(plugin.pluginName, []);
			}
			extensions.get(plugin.pluginName)!.push(plugin);
		}

		const lowerSearchText = searchText.toLowerCase();

		for (const [, commands] of extensions.entries()) {
			const firstCommand = commands[0];
			const hasGlobalPrefs = firstCommand.preferences && firstCommand.preferences.length > 0;
			const commandsWithPrefs = commands.filter(
				(p) => p.commandPreferences && p.commandPreferences.length > 0
			);

			if (!hasGlobalPrefs && commandsWithPrefs.length === 0) continue;

			const extensionMatches =
				firstCommand.pluginTitle.toLowerCase().includes(lowerSearchText) ||
				(firstCommand.description ?? '').toLowerCase().includes(lowerSearchText);

			const matchingCommands = commandsWithPrefs.filter(
				(c) =>
					c.title.toLowerCase().includes(lowerSearchText) ||
					(c.description ?? '').toLowerCase().includes(lowerSearchText)
			);

			if (searchText && !extensionMatches && matchingCommands.length === 0) {
				continue;
			}

			items.push({
				id: firstCommand.pluginName,
				type: 'extension',
				itemType: 'item',
				data: firstCommand,
				isLastInGroup: false
			});

			const commandsToShow = extensionMatches ? commandsWithPrefs : matchingCommands;
			commandsToShow.forEach((command, index) => {
				items.push({
					id: `${command.pluginName}/${command.commandName}`,
					type: 'command',
					itemType: 'item',
					data: command,
					isLastInGroup: index === commandsToShow.length - 1
				});
			});
		}

		return items;
	});

	const selectedItem = $derived(displayItems[selectedIndex]);

	const preferencesToShow = $derived.by(() => {
		if (!selectedItem) return [];
		return selectedItem.type === 'extension'
			? (selectedItem.data.preferences ?? [])
			: (selectedItem.data.commandPreferences ?? []);
	});

	$effect(() => {
		if (selectedItem) {
			onGetPreferences(selectedItem.data.pluginName);
		}
	});

	$effect(() => {
		if (pluginToSelectInSettings) {
			const index = displayItems.findIndex(
				(item) => item.type === 'extension' && item.data.pluginName === pluginToSelectInSettings
			);
			if (index > -1) {
				selectedIndex = index;
			}
		}
	});

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape' && !event.defaultPrevented) {
			event.preventDefault();
			onBack();
		}
	}

	function handleSave() {
		if (selectedItem) {
			onSavePreferences(selectedItem.data.pluginName, preferenceValues);
		}
	}

	function handlePreferenceChange(prefName: string, value: unknown) {
		const newValues = { ...preferenceValues };
		(newValues as Record<string, unknown>)[prefName] = value;
		preferenceValues = newValues;
	}

	function getPreferenceValue(pref: Preference): unknown {
		return (preferenceValues as Record<string, unknown>)[pref.name] ?? pref.default ?? '';
	}

	async function browse(type: 'file' | 'directory', prefName: string) {
		const result = await openDialog({
			directory: type === 'directory',
			multiple: false
		});
		if (typeof result === 'string') {
			handlePreferenceChange(prefName, result);
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="bg-background text-foreground h-screen">
	<Tabs.Root bind:value={activeTab} class="h-full pt-2">
		<Tabs.List class="mx-auto">
			<Tabs.Trigger value="extensions">Extensions</Tabs.Trigger>
			<Tabs.Trigger value="ai">AI</Tabs.Trigger>
		</Tabs.List>
		<Tabs.Content value="ai">
			<AiSettingsView />
		</Tabs.Content>
		<Tabs.Content value="extensions" class="flex h-full">
			<div class="flex w-80 flex-col border-r">
				<header class="mb-2 flex h-15 shrink-0 items-center border-b">
					<button onclick={onBack} class="hover:bg-accent mr-2 rounded p-1">
						<Icon icon="chevron-left-16" class="size-4" />
					</button>
					<Input
						class="rounded-none border-none !bg-transparent"
						placeholder="Filter by name..."
						bind:value={searchText}
					/>
				</header>

				<div class="flex-1 overflow-y-auto">
					<BaseList items={displayItems} bind:selectedIndex onenter={() => {}}>
						{#snippet itemSnippet({ item, isSelected, onclick })}
							{@const assetsPath = path.dirname(item.data.pluginPath) + '/assets'}
							<div class="relative">
								<button
									type="button"
									class="hover:bg-accent/50 flex w-full items-center gap-3 py-3 text-left"
									class:bg-accent={isSelected}
									class:pl-4={item.type === 'extension'}
									class:pl-12={item.type === 'command'}
									{onclick}
								>
									<div class="flex size-8 shrink-0 items-center justify-center">
										<Icon icon={item.data.icon || 'app-window-16'} {assetsPath} class="size-5" />
									</div>
									<div class="flex min-w-0 flex-col">
										<span class="truncate text-sm font-medium">
											{item.type === 'extension' ? item.data.pluginTitle : item.data.title}
										</span>
										<span class="text-muted-foreground truncate text-xs">
											{item.type === 'extension' ? 'Extension' : 'Command'}
										</span>
									</div>
								</button>
							</div>
						{/snippet}
					</BaseList>
				</div>
			</div>

			<div class="flex h-full flex-1 flex-col overflow-y-auto px-4">
				{#if selectedItem}
					<div>
						<div class="mb-6 flex items-start justify-between">
							<div>
								<h2 class="text-lg font-medium">
									{selectedItem.type === 'extension'
										? selectedItem.data.pluginTitle
										: selectedItem.data.title}
								</h2>
								<p class="text-muted-foreground mt-1 text-sm">
									{selectedItem.type === 'extension'
										? 'These settings apply to the entire extension.'
										: 'These settings apply only to this command.'}
								</p>
							</div>

							<Button onclick={handleSave}>Save</Button>
						</div>
						{#if preferencesToShow.length > 0}
							<div class="max-w-md space-y-6">
								{#each preferencesToShow as pref (pref.name)}
									{#if pref.type === 'checkbox'}
										<div class="space-y-2">
											{#if pref.title}
												<div class="text-sm font-medium">
													{pref.title}
													{#if pref.required}<span class="text-red-500">*</span>{/if}
												</div>
											{/if}
											{#if pref.description}
												<p class="text-muted-foreground text-xs">{pref.description}</p>
											{/if}
											<label class="flex items-center gap-2">
												<Checkbox
													checked={getPreferenceValue(pref) as boolean}
													onCheckedChange={(checked) => handlePreferenceChange(pref.name, checked)}
												/>
												<span class="text-sm">{pref.label}</span>
											</label>
										</div>
									{:else}
										<div class="space-y-2">
											<div class="text-sm font-medium">
												{pref.title}
												{#if pref.required}<span class="text-red-500">*</span>{/if}
											</div>

											{#if pref.description}
												<p class="text-muted-foreground text-xs">{pref.description}</p>
											{/if}

											{#if pref.type === 'textfield'}
												<Input
													value={getPreferenceValue(pref) as string}
													onchange={(e) =>
														handlePreferenceChange(
															pref.name,
															(e.target as HTMLInputElement)?.value
														)}
													placeholder={pref.default as string}
												/>
											{:else if pref.type === 'password'}
												<PasswordInput
													value={getPreferenceValue(pref) as string}
													onchange={(e) =>
														handlePreferenceChange(
															pref.name,
															(e.target as HTMLInputElement)?.value
														)}
													placeholder="••••••••••••"
												/>
											{:else if pref.type === 'dropdown' && pref.data}
												<Select.Root
													value={getPreferenceValue(pref) as string}
													onValueChange={(value) => handlePreferenceChange(pref.name, value)}
													type="single"
												>
													<Select.Trigger
														class="bg-background border-border w-full rounded border px-3 py-2 text-sm"
													>
														{@const preference = pref.data.find(
															(option) => option.value === getPreferenceValue(pref)
														)}
														{preference?.title ?? pref.default}
													</Select.Trigger>
													<Select.Content>
														{#each pref.data as option (option.value)}
															<Select.Item value={option.value}>{option.title}</Select.Item>
														{/each}
													</Select.Content>
												</Select.Root>
											{:else if pref.type === 'appPicker'}
												<Select.Root
													value={(getPreferenceValue(pref) as string) || undefined}
													onValueChange={(value) => handlePreferenceChange(pref.name, value)}
													type="single"
												>
													<Select.Trigger class="w-full">
														{@const selectedApp = apps.find(
															(a) => a.exec === getPreferenceValue(pref)
														)}
														{selectedApp?.name ?? 'Select Application'}
													</Select.Trigger>
													<Select.Content>
														{#each apps as app (app.exec)}
															<Select.Item value={app.exec}>{app.name}</Select.Item>
														{/each}
													</Select.Content>
												</Select.Root>
											{:else if pref.type === 'file' || pref.type === 'directory'}
												<div class="flex items-center gap-2">
													<Input
														value={getPreferenceValue(pref) as string}
														onchange={(e) =>
															handlePreferenceChange(
																pref.name,
																(e.target as HTMLInputElement)?.value
															)}
														placeholder={pref.default as string}
														class="flex-grow"
													/>
													<Button
														variant="outline"
														onclick={() => browse(pref.type as 'file' | 'directory', pref.name)}
													>
														Browse...
													</Button>
												</div>
											{/if}
										</div>
									{/if}
								{/each}
							</div>
						{:else}
							<div class="flex h-full items-center justify-center">
								<div class="text-center">
									<p class="text-muted-foreground">No preferences to configure.</p>
								</div>
							</div>
						{/if}
					</div>
				{:else}
					<div class="flex h-full flex-1 items-center justify-center">
						<div class="text-center">
							<p class="text-muted-foreground">Select an item to configure its settings</p>
						</div>
					</div>
				{/if}
			</div>
		</Tabs.Content>
	</Tabs.Root>
</main>
