<script lang="ts">
	import { sidecarService } from '$lib/sidecar.svelte';
	import { uiStore } from '$lib/ui.svelte';
	import SettingsView from '$lib/components/SettingsView.svelte';
	import type { PluginInfo } from '@flare/protocol';
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
	import LogViewer from '$lib/components/LogViewer.svelte';
	import clipboardHistoryCommandIcon from '$lib/assets/command-clipboard-history-1616x16@2x.png?inline';
	import fileSearchCommandIcon from '$lib/assets/command-file-search-1616x16@2x.png?inline';
	import snippetIcon from '$lib/assets/snippets-package-1616x16@2x.png?inline';
	import storeCommandIcon from '$lib/assets/command-store-1616x16@2x.png?inline';
	import quicklinkIcon from '$lib/assets/quicklinks-package-1616x16@2x.png?inline';
	import starsSquareIcon from '$lib/assets/stars-square-1616x16@2x.png?inline';
	import { invoke } from '@tauri-apps/api/core';
	import AiChatView from '$lib/components/AiChatView.svelte';
	import DmenuView from '$lib/components/DmenuView.svelte';
	import DownloadsView from '$lib/components/DownloadsView.svelte';
	import FloatingNotesWindow from '$lib/components/FloatingNotesWindow.svelte';

	const storePlugin: PluginInfo = {
		title: 'Store',
		description: 'Browse and install new extensions from the Store',
		pluginTitle: 'Raycast',
		pluginName: 'raycast',
		commandName: 'store',
		pluginPath: 'builtin:store',
		icon: storeCommandIcon,
		preferences: [],
		mode: 'view',
		owner: 'raycast'
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

	const aiChatPlugin: PluginInfo = {
		title: 'Ask AI',
		description: 'Chat with AI to answer questions, write code, and more',
		pluginTitle: 'AI',
		pluginName: 'ai',
		commandName: 'ask-ai',
		pluginPath: 'builtin:ai-chat',
		icon: starsSquareIcon,
		preferences: [],
		mode: 'view',
		owner: 'flare'
	};

	const downloadsPlugin: PluginInfo = {
		title: 'Downloads',
		description: 'View and manage your recent downloads',
		pluginTitle: 'Flare',
		pluginName: 'downloads',
		commandName: 'downloads',
		pluginPath: 'builtin:downloads',
		icon: fileSearchCommandIcon, // reusing file search icon for now
		preferences: [],
		mode: 'view',
		owner: 'flare'
	};

	const floatingNotesPlugin: PluginInfo = {
		title: 'Toggle Floating Notes',
		description: 'Show/Hide the floating notes window',
		pluginTitle: 'Notes',
		pluginName: 'notes',
		commandName: 'toggle-notes',
		pluginPath: 'builtin:toggle-floating-notes',
		icon: starsSquareIcon, // TODO: Notes icon
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const openLatestDownloadPlugin: PluginInfo = {
		title: 'Open Latest Download',
		description: 'Open the most recent download',
		pluginTitle: 'Downloads',
		pluginName: 'downloads',
		commandName: 'open-latest',
		pluginPath: 'builtin:open-latest-download',
		icon: fileSearchCommandIcon,
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const copyLatestDownloadPlugin: PluginInfo = {
		title: 'Copy Latest Download',
		description: 'Copy the most recent download path to clipboard',
		pluginTitle: 'Downloads',
		pluginName: 'downloads',
		commandName: 'copy-latest',
		pluginPath: 'builtin:copy-latest-download',
		icon: fileSearchCommandIcon,
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const settingsPlugin: PluginInfo = {
		title: 'Flareup Settings',
		description: 'Configure Flareup preferences and options',
		pluginTitle: 'Flare',
		pluginName: 'flare',
		commandName: 'settings',
		pluginPath: 'builtin:settings',
		icon: storeCommandIcon, // reusing store icon for now
		preferences: [],
		mode: 'view',
		owner: 'flare'
	};

	// System Commands
	const lockScreenPlugin: PluginInfo = {
		title: 'Lock Screen',
		description: 'Lock your screen',
		pluginTitle: 'System',
		pluginName: 'system',
		commandName: 'lock-screen',
		pluginPath: 'builtin:lock-screen',
		icon: '', // TODO: Add icon
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const sleepPlugin: PluginInfo = {
		title: 'Sleep',
		description: 'Put your computer to sleep',
		pluginTitle: 'System',
		pluginName: 'system',
		commandName: 'sleep',
		pluginPath: 'builtin:sleep',
		icon: '', // TODO: Add icon
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const shutdownPlugin: PluginInfo = {
		title: 'Shut Down',
		description: 'Shut down your computer',
		pluginTitle: 'System',
		pluginName: 'system',
		commandName: 'shutdown',
		pluginPath: 'builtin:shutdown',
		icon: '', // TODO: Add icon
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const restartPlugin: PluginInfo = {
		title: 'Restart',
		description: 'Restart your computer',
		pluginTitle: 'System',
		pluginName: 'system',
		commandName: 'restart',
		pluginPath: 'builtin:restart',
		icon: '', // TODO: Add icon
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const volumeUpPlugin: PluginInfo = {
		title: 'Volume Up',
		description: 'Increase system volume',
		pluginTitle: 'System',
		pluginName: 'system',
		commandName: 'volume-up',
		pluginPath: 'builtin:volume-up',
		icon: '', // TODO: Add icon
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const volumeDownPlugin: PluginInfo = {
		title: 'Volume Down',
		description: 'Decrease system volume',
		pluginTitle: 'System',
		pluginName: 'system',
		commandName: 'volume-down',
		pluginPath: 'builtin:volume-down',
		icon: '', // TODO: Add icon
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const toggleMutePlugin: PluginInfo = {
		title: 'Toggle Mute',
		description: 'Mute or unmute system audio',
		pluginTitle: 'System',
		pluginName: 'system',
		commandName: 'toggle-mute',
		pluginPath: 'builtin:toggle-mute',
		icon: '', // TODO: Add icon
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const emptyTrashPlugin: PluginInfo = {
		title: 'Empty Trash',
		description: 'Permanently delete all items in trash',
		pluginTitle: 'System',
		pluginName: 'system',
		commandName: 'empty-trash',
		pluginPath: 'builtin:empty-trash',
		icon: '',
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	// Window Management
	const snapLeftPlugin: PluginInfo = {
		title: 'Snap Window Left Half',
		description: 'Move window to left half of screen',
		pluginTitle: 'Window',
		pluginName: 'window',
		commandName: 'snap-left',
		pluginPath: 'builtin:snap-left',
		icon: '',
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const snapRightPlugin: PluginInfo = {
		title: 'Snap Window Right Half',
		description: 'Move window to right half of screen',
		pluginTitle: 'Window',
		pluginName: 'window',
		commandName: 'snap-right',
		pluginPath: 'builtin:snap-right',
		icon: '',
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const snapTopPlugin: PluginInfo = {
		title: 'Snap Window Top Half',
		description: 'Move window to top half of screen',
		pluginTitle: 'Window',
		pluginName: 'window',
		commandName: 'snap-top',
		pluginPath: 'builtin:snap-top',
		icon: '',
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const snapBottomPlugin: PluginInfo = {
		title: 'Snap Window Bottom Half',
		description: 'Move window to bottom half of screen',
		pluginTitle: 'Window',
		pluginName: 'window',
		commandName: 'snap-bottom',
		pluginPath: 'builtin:snap-bottom',
		icon: '',
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const snapTopLeftPlugin: PluginInfo = {
		title: 'Snap Window Top Left',
		description: 'Move window to top left quarter',
		pluginTitle: 'Window',
		pluginName: 'window',
		commandName: 'snap-top-left',
		pluginPath: 'builtin:snap-top-left',
		icon: '',
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const snapTopRightPlugin: PluginInfo = {
		title: 'Snap Window Top Right',
		description: 'Move window to top right quarter',
		pluginTitle: 'Window',
		pluginName: 'window',
		commandName: 'snap-top-right',
		pluginPath: 'builtin:snap-top-right',
		icon: '',
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const snapBottomLeftPlugin: PluginInfo = {
		title: 'Snap Window Bottom Left',
		description: 'Move window to bottom left quarter',
		pluginTitle: 'Window',
		pluginName: 'window',
		commandName: 'snap-bottom-left',
		pluginPath: 'builtin:snap-bottom-left',
		icon: '',
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const snapBottomRightPlugin: PluginInfo = {
		title: 'Snap Window Bottom Right',
		description: 'Move window to bottom right quarter',
		pluginTitle: 'Window',
		pluginName: 'window',
		commandName: 'snap-bottom-right',
		pluginPath: 'builtin:snap-bottom-right',
		icon: '',
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const centerWindowPlugin: PluginInfo = {
		title: 'Center Window',
		description: 'Center window on screen',
		pluginTitle: 'Window',
		pluginName: 'window',
		commandName: 'center',
		pluginPath: 'builtin:center-window',
		icon: '',
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const maximizeWindowPlugin: PluginInfo = {
		title: 'Maximize Window',
		description: 'Maximize window to full screen',
		pluginTitle: 'Window',
		pluginName: 'window',
		commandName: 'maximize',
		pluginPath: 'builtin:maximize-window',
		icon: '',
		preferences: [],
		mode: 'no-view',
		owner: 'flare'
	};

	const almostMaximizePlugin: PluginInfo = {
		title: 'Almost Maximize Window',
		description: 'Maximize window with padding',
		pluginTitle: 'Window',
		pluginName: 'window',
		commandName: 'almost-maximize',
		pluginPath: 'builtin:almost-maximize',
		icon: '',
		preferences: [],
		mode: 'no-view',
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
		fileSearchPlugin,
		aiChatPlugin,
		downloadsPlugin,
		floatingNotesPlugin,
		openLatestDownloadPlugin,
		copyLatestDownloadPlugin,
		settingsPlugin,
		// System commands
		lockScreenPlugin,
		sleepPlugin,
		shutdownPlugin,
		restartPlugin,
		volumeUpPlugin,
		volumeDownPlugin,
		toggleMutePlugin,
		emptyTrashPlugin,
		// Window management
		snapLeftPlugin,
		snapRightPlugin,
		snapTopPlugin,
		snapBottomPlugin,
		snapTopLeftPlugin,
		snapTopRightPlugin,
		snapBottomLeftPlugin,
		snapBottomRightPlugin,
		centerWindowPlugin,
		maximizeWindowPlugin,
		almostMaximizePlugin
	]);

	const {
		currentView,
		oauthState,
		oauthStatus,
		quicklinkToEdit,
		snippetToEdit,
		snippetsForImport,
		commandToConfirm
	} = $derived(viewManager);

	let showLogViewer = $state(false);
	let dmenuMode = $state(false);
	let isFloatingNotesWindow = $state(false);

	onMount(() => {
		// Check if we are the floating notes window
		if (window.location.pathname === '/floating-notes') {
			isFloatingNotesWindow = true;
			return;
		}

		// Listen for dmenu mode activation FIRST (before any other init)
		const unlistenDmenu = listen('dmenu-mode', () => {
			dmenuMode = true;
		});

		// If we're in dmenu mode, skip normal app initialization
		// The dmenu event may have already been emitted, so we need to query for it
		invoke('dmenu_get_items')
			.then((items: unknown) => {
				// If this succeeds, we're in dmenu mode
				if (Array.isArray(items) && items.length >= 0) {
					dmenuMode = true;
				}
			})
			.catch(() => {
				// Not in dmenu mode - proceed with normal initialization
				initNormalMode();
			});

		return () => {
			sidecarService.stop();
			unlistenDmenu.then((fn) => fn());
		};
	});

	function initNormalMode() {
		sidecarService.setOnGoBackToPluginList(viewManager.showCommandPalette);
		sidecarService.start();

		invoke<PluginInfo[]>('get_discovered_plugins')
			.then((plugins) => {
				uiStore.setPluginList(plugins);
			})
			.catch((e) => {
				console.error('Failed to discover plugins:', e);
			});

		listen<string>('deep-link', (event) => {
			viewManager.handleDeepLink(event.payload, allPlugins);
		});

		// Listen for hotkey-triggered commands
		listen<string>('execute-command', (event) => {
			const commandId = event.payload;

			// Find the plugin for this command
			const plugin = allPlugins.find((p) => p.pluginPath === commandId);
			if (plugin) {
				viewManager.runPlugin(plugin);
			} else {
				console.error('[Hotkey] Command not found:', commandId);
			}
		});
	}

	$effect(() => {
		viewManager.oauthState = sidecarService.oauthState;
	});

	$effect(() => {
		if (oauthStatus === 'authorizing' && oauthState?.url) {
			openUrl(oauthState.url);
		}
	});

	function handleKeydown(event: KeyboardEvent) {
		// Toggle log viewer with Cmd/Ctrl + Shift + L
		if (event.key === 'L' && (event.metaKey || event.ctrlKey) && event.shiftKey) {
			event.preventDefault();
			showLogViewer = !showLogViewer;
			return;
		}

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

{#if isFloatingNotesWindow}
	<FloatingNotesWindow />
{:else if dmenuMode}
	<DmenuView />
{:else if currentView === 'command-palette'}
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
	<SearchSnippets onBack={viewManager.showCommandPalette} onEdit={viewManager.showSnippetForm} />
{:else if currentView === 'quicklink-form'}
	<QuicklinkForm
		quicklink={quicklinkToEdit}
		onBack={viewManager.showCommandPalette}
		onSave={viewManager.showCommandPalette}
	/>
{:else if currentView === 'create-snippet-form'}
	<SnippetForm
		editSnippet={snippetToEdit}
		onBack={viewManager.showSearchSnippets}
		onSave={viewManager.showSearchSnippets}
	/>
{:else if currentView === 'import-snippets'}
	<ImportSnippets onBack={viewManager.showCommandPalette} snippetsToImport={snippetsForImport} />
{:else if currentView === 'file-search'}
	<FileSearchView onBack={viewManager.showCommandPalette} />
{:else if currentView === 'ai-chat'}
	<AiChatView onBack={viewManager.showCommandPalette} />
{:else if currentView === 'downloads'}
	<DownloadsView onBack={viewManager.showCommandPalette} />
{/if}

{#if showLogViewer}
	<LogViewer onClose={() => (showLogViewer = false)} />
{/if}
