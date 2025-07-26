import type { PluginInfo } from '@flare/protocol';
import { invoke } from '@tauri-apps/api/core';
import Fuse from 'fuse.js';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import type { Quicklink } from '$lib/quicklinks.svelte';
import { frecencyStore } from './frecency.svelte';
import { viewManager } from './viewManager.svelte';
import type { App } from './apps.svelte';

export type UnifiedItem = {
	type: 'calculator' | 'plugin' | 'app' | 'quicklink';
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
		const items: { type: 'plugin' | 'app' | 'quicklink'; id: string; data: any }[] = [];
		items.push(...plugins().map((p) => ({ type: 'plugin', id: p.pluginPath, data: p }) as const));
		items.push(...installedApps().map((a) => ({ type: 'app', id: a.exec, data: a }) as const));
		items.push(
			...quicklinks().map((q) => ({ type: 'quicklink', id: `quicklink-${q.id}`, data: q }) as const)
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
		} else {
			items = allSearchableItems.map((item) => ({ ...item, score: 0, fuseScore: 1 }));
		}

		const frecencyMap = new Map(frecencyData().map((item) => [item.itemId, item]));
		const now = Date.now() / 1000;
		const gravity = 1.8;

		items.forEach((item) => {
			const frecency = frecencyMap.get(item.id);
			let frecencyScore = 0;
			if (frecency) {
				const ageInHours = Math.max(1, (now - frecency.lastUsedAt) / 3600);
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

		await frecencyStore.recordUsage(item.id);

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

	return {
		executeQuicklink,
		handleEnter,
		handleResetRanking,
		handleCopyDeeplink,
		handleConfigureCommand,
		handleCopyAppName,
		handleCopyAppPath,
		handleHideApp
	};
}
