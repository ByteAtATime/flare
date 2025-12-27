import type { PluginInfo } from '@flare/protocol';
import { invoke } from '@tauri-apps/api/core';
import Fuse from 'fuse.js';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import type { Quicklink } from '$lib/quicklinks.svelte';
import { frecencyStore } from './frecency.svelte';
import { viewManager } from './viewManager.svelte';
import type { App } from './apps.svelte';
import { aiStore, type AiPreset } from './ai.svelte';
import { aliasesStore } from './aliases.svelte';
import { scriptCommandsStore, type ScriptCommand } from './script-commands.svelte';

export type UnifiedItem = {
	type:
		| 'calculator'
		| 'plugin'
		| 'app'
		| 'quicklink'
		| 'ai-preset'
		| 'ask-ai'
		| 'script-command';
	id: string;
	data: any;
	score: number;
};

type UseCommandPaletteItemsArgs = {
	searchText: () => string;
	plugins: () => PluginInfo[];
	installedApps: () => App[];
	quicklinks: () => Quicklink[];
	frecencyData: () => { itemId: string; useCount: number; lastUsedAt: number }[];
	selectedQuicklinkForArgument: () => Quicklink | null;
};

export function useCommandPaletteItems({
	searchText,
	plugins,
	installedApps,
	quicklinks,
	frecencyData,
	selectedQuicklinkForArgument
}: UseCommandPaletteItemsArgs) {
	const allSearchableItems = $derived.by(() => {
		const items: {
			type: 'plugin' | 'app' | 'quicklink' | 'ai-preset' | 'script-command';
			id: string;
			data: any;
		}[] = [];
		items.push(...plugins().map((p) => ({ type: 'plugin', id: p.pluginPath, data: p }) as const));
		items.push(...installedApps().map((a) => ({ type: 'app', id: a.exec, data: a }) as const));
		items.push(
			...quicklinks().map((q) => ({ type: 'quicklink', id: `quicklink-${q.id}`, data: q }) as const)
		);
		items.push(
			...aiStore.presets.map(
				(p) => ({ type: 'ai-preset', id: `ai-preset-${p.id}`, data: p }) as const
			)
		);
		items.push(
			...scriptCommandsStore.commands.map(
				(c) =>
					({
						type: 'script-command',
						id: `script-${c.path}`,
						data: c
					}) as const
			)
		);
		return items;
	});

	const fuse = $derived(
		new Fuse(allSearchableItems, {
			keys: [
				'data.title',
				'data.pluginTitle',
				'data.description',
				'data.name',
				'data.comment',
				'data.link'
			],
			threshold: 0.4,
			includeScore: true
		})
	);

	let calculatorResult = $state<{ value: string; type: string } | null>(null);
	let calculationId = 0;

	$effect(() => {
		const term = searchText();
		calculationId++;
		const currentCalculationId = calculationId;

		if (!term.trim() || selectedQuicklinkForArgument()) {
			calculatorResult = null;
			return;
		}

		(async () => {
			try {
				const resultJson = await invoke<string>('calculate_soulver', { expression: term.trim() });

				if (currentCalculationId !== calculationId) {
					return; // Stale request
				}

				const result = JSON.parse(resultJson) as {
					value: string;
					type: string;
					error?: string;
				};

				if (result.error) {
					console.error('Soulver error:', result.error);
					calculatorResult = null;
					return;
				}

				if (result.type === 'none' || !result.value) {
					calculatorResult = null;
					return;
				}

				if (result.value === term.trim()) {
					calculatorResult = null;
					return;
				}

				calculatorResult = { value: result.value, type: result.type };
			} catch (e) {
				if (currentCalculationId !== calculationId) {
					return; // Stale request
				}
				console.error('Soulver invocation failed:', e);
				calculatorResult = null;
			}
		})();
	});

	const displayItems = $derived.by(() => {
		let items: (UnifiedItem & { fuseScore?: number })[] = [];
		const term = searchText();

		if (term.trim()) {
			items = fuse.search(term).map((result) => ({
				...result.item,
				score: 0,
				fuseScore: result.score
			}));

			// Add "Ask AI" fallback
			items.push({
				type: 'ask-ai',
				id: 'ask-ai',
				data: { query: term },
				score: 0.1, // Low score to be at the bottom usually, or adjust based on preference
				fuseScore: 1
			});
		} else {
			items = allSearchableItems.map((item) => ({ ...item, score: 0, fuseScore: 1 }));
		}

		const frecencyMap = new Map(frecencyData().map((item) => [item.itemId, item]));
		const now = Date.now() / 1000;
		const gravity = 1.8;

		const aliasMatch = aliasesStore.getCommandId(term.trim());

		items.forEach((item) => {
			// Check for alias match
			if (aliasMatch && item.id === aliasMatch) {
				item.score = 10000; // Massive boost for alias match
				return;
			}

			const frecency = frecencyMap.get(item.id);
			let frecencyScore = 0;
			if (frecency) {
				// Backend stores timestamp in nanoseconds, convert to seconds
				const lastUsedSeconds = frecency.lastUsedAt / 1_000_000_000;
				const nowSeconds = Date.now() / 1000;
				const ageInHours = Math.max(1, (nowSeconds - lastUsedSeconds) / 3600);
				frecencyScore = (frecency.useCount * 1000) / Math.pow(ageInHours + 2, gravity);
			}
			const textScore = item.fuseScore !== undefined ? 1 - item.fuseScore * 100 : 0;
			item.score = frecencyScore + textScore;
		});

		items.sort((a, b) => b.score - a.score);

		const calcRes = calculatorResult;
		if (calcRes) {
			items.unshift({
				type: 'calculator',
				id: 'calculator',
				data: {
					value: term,
					result: calcRes.value,
					resultType: calcRes.type
				},
				score: 9999
			});
		}

		return [...new Map(items.map((item) => [item.id, item])).values()];
	});

	return () => ({
		displayItems
	});
}

type UseCommandPaletteActionsArgs = {
	selectedItem: () => UnifiedItem | undefined;
	onRunPlugin: (plugin: PluginInfo) => void;
	resetState: () => void;
	focusArgumentInput: () => void;
};

export function useCommandPaletteActions({
	selectedItem,
	onRunPlugin,
	resetState,
	focusArgumentInput
}: UseCommandPaletteActionsArgs) {
	async function executeQuicklink(quicklink: Quicklink, argument?: string) {
		const finalLink = argument
			? quicklink.link.replace(/\{argument\}/g, encodeURIComponent(argument))
			: quicklink.link.replace(/\{argument\}/g, '');
		await invoke('execute_quicklink', {
			link: finalLink,
			application: quicklink.application
		});
		resetState();
	}

	async function handleEnter() {
		const item = selectedItem();
		if (!item) return;

		// Don't record usage for calculator or ask-ai dynamic items if you prefer
		if (item.type !== 'calculator' && item.type !== 'ask-ai') {
			await frecencyStore.recordUsage(item.id);
		}

		switch (item.type) {
			case 'calculator': {
				writeText(item.data.result);
				break;
			}
			case 'plugin': {
				onRunPlugin(item.data as PluginInfo);
				break;
			}
			case 'app': {
				if (item.data.exec) {
					invoke('launch_app', { exec: item.data.exec }).catch(console.error);
				}
				break;
			}
			case 'quicklink': {
				const quicklink = item.data as Quicklink;
				if (quicklink.link.includes('{argument}')) {
					focusArgumentInput();
				} else {
					executeQuicklink(quicklink);
				}
				break;
			}
			case 'ai-preset': {
				const preset = item.data as AiPreset;
				try {
					const selection = await invoke<string>('get_selected_text');
					const prompt = preset.template.replace('{selection}', selection);
					viewManager.showAiChat(prompt);
				} catch (e) {
					console.error('Failed to get selection for AI preset:', e);
					// If no selection, just open chat with template or blank?
					// For now let's open with template
					viewManager.showAiChat(preset.template);
				}
				break;
			}
			case 'ask-ai': {
				viewManager.showAiChat(item.data.query);
				break;
			}
			case 'script-command': {
				const command = item.data as ScriptCommand;
				// TODO: Handle arguments
				// For now, assume no args or handle interactively
                if (command.arguments.length > 0) {
                    // Logic to prompt for args would go here
                    // Similar to quicklinks with arguments
                    console.log('Script has arguments, not fully implemented in UI yet');
                } else {
                    const result = await scriptCommandsStore.runCommand(command, []);
                    if (command.mode === 'fullOutput') {
                        // Show output view
                        // viewManager.showScriptOutput(result);
						// For MVP, just use HUD or clipboard if needed, or simple alert
						alert(result);
                    } else if (command.mode === 'compact') {
                        await invoke('show_hud', { title: result });
                    } else if (command.mode === 'inline') {
                        await invoke('clipboard_paste', { text: result });
                    }
                }
				break;
			}
		}
	}

	async function handleResetRanking() {
		const item = selectedItem();
		if (item) {
			await frecencyStore.deleteEntry(item.id);
		}
	}

	function handleCopyDeeplink() {
		const item = selectedItem();
		if (item?.type !== 'plugin') return;
		const plugin = item.data as PluginInfo;
		const authorOrOwner =
			plugin.owner === 'raycast'
				? 'raycast'
				: typeof plugin.author === 'string'
					? plugin.author
					: (plugin.author?.name ?? 'unknown');

		const deeplink = `raycast://extensions/${authorOrOwner}/${plugin.pluginName}/${plugin.commandName}`;
		writeText(deeplink);
	}

	function handleConfigureCommand() {
		const item = selectedItem();
		if (item?.type !== 'plugin') return;
		viewManager.showSettings(item.data.pluginName);
	}

	function handleCopyAppName() {
		const item = selectedItem();
		if (item?.type !== 'app') return;
		writeText(item.data.name);
	}

	function handleCopyAppPath() {
		const item = selectedItem();
		if (item?.type !== 'app') return;
		writeText(item.data.exec);
	}

	async function handleHideApp() {
		const item = selectedItem();
		if (item?.type !== 'app') return;
		await frecencyStore.hideItem(item.id);
	}

	async function handleSetAlias(alias: string) {
		const item = selectedItem();
		if (!item) return;
		await aliasesStore.setAlias(alias, item.id);
	}

	return {
		executeQuicklink,
		handleEnter,
		handleResetRanking,
		handleCopyDeeplink,
		handleConfigureCommand,
		handleCopyAppName,
		handleCopyAppPath,
		handleHideApp,
		handleSetAlias
	};
}
