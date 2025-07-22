<script lang="ts">
	import { sidecarService } from '$lib/sidecar.svelte';
	import { uiStore } from '$lib/ui.svelte';
	import SettingsView from '$lib/components/SettingsView.svelte';
	import type { PluginInfo } from '@raycast-linux/protocol';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import CommandPalette from '$lib/components/command-palette/CommandPalette.svelte';
	import PluginRunner from '$lib/components/PluginRunner.svelte';
	import Extensions from '$lib/components/Extensions.svelte';
	import OAuthView from '$lib/components/OAuthView.svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import ClipboardHistoryView from '$lib/components/ClipboardHistoryView.svelte';
	import QuicklinkForm from '$lib/components/QuicklinkForm.svelte';
	import { viewManager } from '$lib/viewManager.svelte';
	import SnippetForm from '$lib/components/SnippetForm.svelte';
	import ImportSnippets from '$lib/components/ImportSnippets.svelte';
	import SearchSnippets from '$lib/components/SearchSnippets.svelte';
	import FileSearchView from '$lib/components/FileSearchView.svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import CommandDeeplinkConfirm from '$lib/components/CommandDeeplinkConfirm.svelte';
	import clipboardHistoryCommandIcon from '$lib/assets/command-clipboard-history-1616x16@2x.png?inline';
	import fileSearchCommandIcon from '$lib/assets/command-file-search-1616x16@2x.png?inline';
	import snippetIcon from '$lib/assets/snippets-package-1616x16@2x.png?inline';
	import storeCommandIcon from '$lib/assets/command-store-1616x16@2x.png?inline';
	import quicklinkIcon from '$lib/assets/quicklinks-package-1616x16@2x.png?inline';
	import { invoke } from '@tauri-apps/api/core';

	const storePlugin: PluginInfo = {
		title: 'Store',
		description: 'Browse and install new extensions from the Store',
		pluginTitle: 'Flare',
		pluginName: 'flare',
		commandName: 'store',
		pluginPath: 'builtin:store',
		icon: storeCommandIcon,
		preferences: [],
		mode: 'view',
		owner: 'flare'
	};

	const clipboardHistoryPlugin: PluginInfo = {
		title: 'Clipboard History',
		description: 'View, search, and manage your clipboard history',
		pluginTitle: 'Flare',
		pluginName: 'clipboard-history',
		commandName: 'clipboard-history',
		pluginPath: 'builtin:history',
		icon: clipboardHistoryCommandIcon,
		preferences: [],
		mode: 'view',
		owner: 'flare'
	};

	const searchSnippetsPlugin: PluginInfo = {
		title: 'Search Snippets',
		description: 'Search and manage your snippets',
		pluginTitle: 'Snippets',
		pluginName: 'snippets',
		commandName: 'search-snippets',
		pluginPath: 'builtin:search-snippets',
		icon: snippetIcon,
		preferences: [],
		mode: 'view',
		owner: 'flare'
	};

	const createQuicklinkPlugin: PluginInfo = {
		title: 'Create Quicklink',
		description: 'Create a new Quicklink',
		pluginTitle: 'Flare',
		pluginName: 'flare',
		commandName: 'create-quicklink',
		pluginPath: 'builtin:create-quicklink',
		icon: quicklinkIcon,
		preferences: [],
		mode: 'view',
		owner: 'flare'
	};

	const createSnippetPlugin: PluginInfo = {
		title: 'Create Snippet',
		description: 'Create a new snippet',
		pluginTitle: 'Flare',
		pluginName: 'snippets',
		commandName: 'create-snippet',
		pluginPath: 'builtin:create-snippet',
		icon: snippetIcon,
		preferences: [],
		mode: 'view',
		owner: 'flare'
	};

	const importSnippetsPlugin: PluginInfo = {
		title: 'Import Snippets',
		description: 'Import snippets from a JSON file',
		pluginTitle: 'Flare',
		pluginName: 'snippets',
		commandName: 'import-snippets',
		pluginPath: 'builtin:import-snippets',
		icon: snippetIcon,
		preferences: [],
		mode: 'view',
		owner: 'flare'
	};

	const fileSearchPlugin: PluginInfo = {
		title: 'Search Files',
		description: 'Find files and folders on your computer',
		pluginTitle: 'Flare',
		pluginName: 'file-search',
		commandName: 'search-files',
		pluginPath: 'builtin:file-search',
		icon: fileSearchCommandIcon,
		preferences: [],
		mode: 'view',
		owner: 'flare'
	};

	const { pluginList, currentPreferences } = $derived(uiStore);
	const allPlugins = $derived([
		...pluginList,
		storePlugin,
		clipboardHistoryPlugin,
		searchSnippetsPlugin,
		createQuicklinkPlugin,
		createSnippetPlugin,
		importSnippetsPlugin,
		fileSearchPlugin
	]);

	const {
		currentView,
		oauthState,
		oauthStatus,
		quicklinkToEdit,
		snippetsForImport,
		commandToConfirm
	} = $derived(viewManager);

	onMount(() => {
		sidecarService.setOnGoBackToPluginList(viewManager.showCommandPalette);
		sidecarService.start();

		invoke<PluginInfo[]>('get_discovered_plugins')
			.then((plugins) => {
				uiStore.setPluginList(plugins);
			})
			.catch((e) => {
				console.error('Failed to discover plugins:', e);
			});

		const unlisten = listen<string>('deep-link', (event) => {
			console.log('Received deep link:', event.payload);
			viewManager.handleDeepLink(event.payload, allPlugins);
		});

		return () => {
			sidecarService.stop();
			unlisten.then((fn) => fn());
		};
	});

	$effect(() => {
		viewManager.oauthState = sidecarService.oauthState;
	});

	$effect(() => {
		if (oauthStatus === 'authorizing' && oauthState?.url) {
			openUrl(oauthState.url);
		}
	});

	function handleKeydown(event: KeyboardEvent) {
		if (
			currentView === 'command-palette' &&
			event.key === ',' &&
			(event.metaKey || event.ctrlKey)
		) {
			event.preventDefault();
			viewManager.showSettings();
			return;
		}

		if (event.key === 'Escape') {
			if (currentView === 'command-palette' && !event.defaultPrevented) {
				event.preventDefault();
				getCurrentWindow().hide();
			}
		}
	}

	function handleSavePreferences(pluginName: string, values: Record<string, unknown>) {
		sidecarService.setPreferences(pluginName, values);
	}

	function handleGetPreferences(pluginName: string) {
		sidecarService.getPreferences(pluginName);
	}

	function handlePopView() {
		sidecarService.dispatchEvent('pop-view');
	}

	function handleToastAction(toastId: number, actionType: 'primary' | 'secondary') {
		sidecarService.dispatchEvent('dispatch-toast-action', { toastId, actionType });
	}

	function onExtensionInstalled() {
		invoke<PluginInfo[]>('get_discovered_plugins')
			.then((plugins) => {
				uiStore.setPluginList(plugins);
			})
			.catch((e) => {
				console.error('Failed to discover plugins:', e);
			});
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if commandToConfirm}
	<CommandDeeplinkConfirm
		plugin={commandToConfirm}
		onconfirm={viewManager.confirmRunCommand}
		oncancel={viewManager.cancelRunCommand}
	/>
{/if}

{#if oauthState}
	<OAuthView
		providerName={oauthState.providerName}
		providerIcon={oauthState.providerIcon}
		description={oauthState.description}
		authUrl={oauthState.url}
		status={oauthStatus}
		onSignIn={viewManager.handleOauthSignIn}
		onBack={() => (sidecarService.oauthState = null)}
	/>
{/if}

{#if currentView === 'command-palette'}
	<CommandPalette plugins={allPlugins} onRunPlugin={viewManager.runPlugin} />
{:else if currentView === 'settings'}
	<SettingsView
		plugins={pluginList}
		onBack={viewManager.showCommandPalette}
		onSavePreferences={handleSavePreferences}
		onGetPreferences={handleGetPreferences}
		{currentPreferences}
	/>
{:else if currentView === 'extensions-store'}
	<Extensions onBack={viewManager.showCommandPalette} onInstall={onExtensionInstalled} />
{:else if currentView === 'plugin-running'}
	{#key uiStore.currentRunningPlugin?.pluginPath}
		<PluginRunner onPopView={handlePopView} onToastAction={handleToastAction} />
	{/key}
{:else if currentView === 'clipboard-history'}
	<ClipboardHistoryView onBack={viewManager.showCommandPalette} />
{:else if currentView === 'search-snippets'}
	<SearchSnippets onBack={viewManager.showCommandPalette} />
{:else if currentView === 'quicklink-form'}
	<QuicklinkForm
		quicklink={quicklinkToEdit}
		onBack={viewManager.showCommandPalette}
		onSave={viewManager.showCommandPalette}
	/>
{:else if currentView === 'create-snippet-form'}
	<SnippetForm onBack={viewManager.showCommandPalette} onSave={viewManager.showCommandPalette} />
{:else if currentView === 'import-snippets'}
	<ImportSnippets onBack={viewManager.showCommandPalette} snippetsToImport={snippetsForImport} />
{:else if currentView === 'file-search'}
	<FileSearchView onBack={viewManager.showCommandPalette} />
{/if}
