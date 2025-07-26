import type { ImageLike, PluginInfo } from '@flare/protocol';
import { uiStore } from '$lib/ui.svelte';
import { sidecarService } from '$lib/sidecar.svelte';
import type { Quicklink } from './quicklinks.svelte';
import { invoke } from '@tauri-apps/api/core';
import { extensionsStore } from './components/extensions/store.svelte';
import { fetch } from '@tauri-apps/plugin-http';
import { ExtensionSchema, type Extension } from '$lib/store';

export type ViewState =
	| 'command-palette'
	| 'plugin-running'
	| 'settings'
	| 'extensions-store'
	| 'clipboard-history'
	| 'search-snippets'
	| 'quicklink-form'
	| 'create-snippet-form'
	| 'import-snippets'
	| 'file-search';

type OauthState = {
	url: string;
	providerName: string;
	providerIcon?: ImageLike;
	description?: string;
} | null;

class ViewManager {
	currentView = $state<ViewState>('command-palette');
	quicklinkToEdit = $state<Quicklink | undefined>(undefined);
	snippetsForImport = $state<any[] | null>(null);
	commandToConfirm = $state<PluginInfo | null>(null);
	pluginToSelectInSettings = $state<string | undefined>(undefined);
	extensionToSelect = $state<Extension | null>(null);

	oauthState: OauthState = $state(null);
	oauthStatus: 'initial' | 'authorizing' | 'success' | 'error' = $state('initial');

	showCommandPalette = () => {
		this.currentView = 'command-palette';
		uiStore.setCurrentRunningPlugin(null);
		this.snippetsForImport = null;
		this.commandToConfirm = null;
		this.pluginToSelectInSettings = undefined;
	};

	showSettings = (pluginName?: string) => {
		this.currentView = 'settings';
		this.pluginToSelectInSettings = pluginName;
	};

	showExtensions = (extension?: Extension) => {
		this.currentView = 'extensions-store';
		this.extensionToSelect = extension ?? null;
	};

	showClipboardHistory = () => {
		this.currentView = 'clipboard-history';
	};

	showSearchSnippets = () => {
		this.currentView = 'search-snippets';
	};

	showQuicklinkForm = (quicklink?: Quicklink) => {
		this.quicklinkToEdit = quicklink;
		this.currentView = 'quicklink-form';
	};

	showCreateSnippetForm = () => {
		this.currentView = 'create-snippet-form';
	};

	showImportSnippets = (snippets?: any[]) => {
		this.snippetsForImport = snippets ?? null;
		this.currentView = 'import-snippets';
	};

	showFileSearch = () => {
		this.currentView = 'file-search';
	};

	runPlugin = async (plugin: PluginInfo) => {
		switch (plugin.pluginPath) {
			case 'builtin:store':
				this.showExtensions();
				return;
			case 'builtin:history':
				this.showClipboardHistory();
				return;
			case 'builtin:search-snippets':
				this.showSearchSnippets();
				return;
			case 'builtin:create-quicklink':
				this.showQuicklinkForm();
				return;
			case 'builtin:create-snippet':
				this.showCreateSnippetForm();
				return;
			case 'builtin:import-snippets':
				this.showImportSnippets();
				return;
			case 'builtin:file-search':
				this.showFileSearch();
				return;
		}

		uiStore.setCurrentRunningPlugin(plugin);

		const hasAiAccess = await invoke<boolean>('ai_can_access');

		sidecarService.dispatchEvent('run-plugin', {
			pluginPath: plugin.pluginPath,
			commandName: plugin.commandName,
			mode: plugin.mode,
			aiAccessStatus: hasAiAccess
		});

		if (plugin.mode !== 'no-view') {
			uiStore.resetForNewPlugin();
			this.currentView = 'plugin-running';
		}
	};

	handleOauthSignIn = () => {
		if (this.oauthState?.url) {
			this.oauthStatus = 'authorizing';
		}
	};

	handleDeepLink = async (url: string, allPlugins: PluginInfo[]) => {
		try {
			const urlObj = new URL(url);
			if (urlObj.protocol === 'raycast:') {
				if (urlObj.host === 'extensions') {
					const parts = urlObj.pathname.split('/').filter(Boolean);
					if (parts.length === 2) {
						const [author, extensionSlug] = parts;

						this.showExtensions();
						extensionsStore.isLoading = true;

						try {
							const res = await fetch(
								`https://backend.raycast.com/api/v1/extensions/${author}/${extensionSlug}`
							);
							if (!res.ok) throw new Error(`Search failed: ${res.status}`);
							const parsed = ExtensionSchema.parse(await res.json());

							this.extensionToSelect = parsed;
						} catch (e) {
							console.error('Failed to fetch extension from deeplink', e);
							extensionsStore.searchText = extensionSlug;
						} finally {
							extensionsStore.isLoading = false;
						}
						return;
					}
					if (parts.length === 3) {
						const [authorOrOwner, extensionName, commandName] = parts;

						const foundPlugin = allPlugins.find((p) => {
							if (authorOrOwner === 'raycast') {
								return (
									p.owner === 'raycast' &&
									p.pluginName === extensionName &&
									p.commandName === commandName
								);
							} else {
								const authorMatch =
									(typeof p.author === 'string' && p.author === authorOrOwner) ||
									(typeof p.author === 'object' && p.author?.name === authorOrOwner);
								const ownerMatch = p.owner === authorOrOwner;
								return (
									(authorMatch || ownerMatch) &&
									p.pluginName === extensionName &&
									p.commandName === commandName
								);
							}
						});

						if (foundPlugin) {
							this.commandToConfirm = foundPlugin;
						} else {
							console.error('Command from deeplink not found:', url);
						}
					}
				} else if (urlObj.host === 'snippets' && urlObj.pathname === '/import') {
					const snippetParams = urlObj.searchParams.getAll('snippet');
					const snippets = snippetParams
						.map((param) => {
							try {
								return JSON.parse(decodeURIComponent(param));
							} catch (e) {
								console.error('Failed to parse snippet JSON:', e);
								return null;
							}
						})
						.filter((s): s is object => s !== null);

					this.showImportSnippets(snippets.length > 0 ? snippets : undefined);
				} else if (urlObj.host === 'oauth' || urlObj.pathname.startsWith('/redirect')) {
					const params = urlObj.searchParams;
					const code = params.get('code');
					const state = params.get('state');

					if (this.oauthState) {
						this.oauthStatus = 'success';
						setTimeout(() => {
							this.oauthState = null;
							this.oauthStatus = 'initial';
						}, 2000);
					}

					if (code && state) {
						sidecarService.dispatchEvent('oauth-authorize-response', { code, state });
					} else {
						const error = params.get('error') || 'Unknown OAuth error';
						const errorDescription = params.get('error_description');
						sidecarService.dispatchEvent('oauth-authorize-response', {
							state,
							error: `${error}: ${errorDescription}`
						});
					}
				} else {
					switch (urlObj.host) {
						case 'extensions':
							this.showExtensions();
							break;
						default:
							this.showCommandPalette();
					}
				}
			}
		} catch (error) {
			console.error('Error parsing deep link:', error);
			this.showCommandPalette();
		}
	};

	confirmRunCommand = () => {
		if (this.commandToConfirm) {
			this.runPlugin(this.commandToConfirm);
			this.commandToConfirm = null;
		}
	};

	cancelRunCommand = () => {
		this.commandToConfirm = null;
	};
}

export const viewManager = new ViewManager();
