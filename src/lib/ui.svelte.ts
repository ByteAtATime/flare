import type { UINode } from '$lib/types';
import type { Command } from '@flare/protocol';
import type { PluginInfo } from '@flare/protocol';
import type { KeyboardShortcut } from '$lib/props/actions';
import { SvelteMap } from 'svelte/reactivity';

export type Toast = {
	id: number;
	title: string;
	message?: string;
	style?: 'SUCCESS' | 'FAILURE' | 'ANIMATED';
	primaryAction?: { title: string; onAction: () => void; shortcut?: KeyboardShortcut };
	secondaryAction?: { title: string; onAction: () => void; shortcut?: KeyboardShortcut };
};

function createUiStore() {
	let uiTree = $state(new Map<number, UINode>());
	let rootNodeId = $state<number | null>(null);
	let selectedNodeId = $state<number | undefined>(undefined);
	let pluginList = $state<PluginInfo[]>([]);
	let currentPreferences = $state<Record<string, unknown>>({});
	let currentRunningPlugin = $state<PluginInfo | null>(null);
	const toasts = new SvelteMap<number, Toast>();
	const propTemplates = new Map<
		number,
		{ props: Record<string, unknown>; namedChildren?: Record<string, number> }
	>();

	const rootNode = $derived(uiTree.get(rootNodeId!));
	const selectedItemNode = $derived(uiTree.get(selectedNodeId!));

	const actionInfo = $derived.by(() => {
		const actionsNodeId =
			rootNode?.type === 'Detail' || rootNode?.type === 'Form'
				? rootNode.namedChildren?.['actions']
				: selectedItemNode?.namedChildren?.['actions'];
		if (!actionsNodeId)
			return { primary: undefined, secondary: undefined, panel: undefined, allActions: [] };
		const panelNode = uiTree.get(actionsNodeId);
		if (!panelNode || panelNode.type !== 'ActionPanel')
			return { primary: undefined, secondary: undefined, panel: undefined, allActions: [] };

		const foundActions: UINode[] = [];
		function findActions(nodeId: number) {
			const node = uiTree.get(nodeId);
			if (!node) return;
			if (
				(node.type.startsWith('Action.') || node.type === 'Action') &&
				!node.type.includes('Panel')
			) {
				foundActions.push(node);
			} else if (node.type.includes('Panel')) {
				for (const childId of node.children) findActions(childId);
			}
		}
		findActions(actionsNodeId);
		return {
			primary: foundActions[0],
			secondary: foundActions[1],
			panel: panelNode,
			allActions: foundActions
		};
	});

	const applyCommands = (commands: Command[]) => {
		const tempTree = new Map(uiTree);
		const tempState = { rootNodeId };

		const mutatedIds = new Set<number>();

		const getMutableNode = (id: number): UINode | undefined => {
			if (id === null || id === undefined) return undefined;
			if (mutatedIds.has(id)) {
				return tempTree.get(id);
			}

			const originalNode = tempTree.get(id);
			if (!originalNode) return undefined;

			const clonedNode = {
				...originalNode,
				props: { ...originalNode.props },
				children: [...originalNode.children]
			};
			tempTree.set(id, clonedNode);
			mutatedIds.add(id);
			return clonedNode;
		};

		for (const command of commands) {
			processSingleCommand(command, tempTree, tempState, getMutableNode);
		}

		uiTree = tempTree;
		rootNodeId = tempState.rootNodeId;
	};

	const setPluginList = (plugins: PluginInfo[]) => {
		pluginList = plugins;
	};

	const setCurrentPreferences = (preferences: Record<string, unknown>) => {
		currentPreferences = preferences;
	};

	const resetForNewPlugin = () => {
		uiTree = new Map();
		rootNodeId = null;
		selectedNodeId = undefined;
		propTemplates.clear();
	};

	function processSingleCommand(
		command: Command,
		tempTree: Map<number, UINode>,
		tempState: { rootNodeId: number | null },
		getMutableNode: (id: number) => UINode | undefined
	) {
		switch (command.type) {
			case 'DEFINE_PROPS_TEMPLATE': {
				const { templateId, props, namedChildren } = command.payload;
				propTemplates.set(templateId, { props, namedChildren });
				break;
			}
			case 'APPLY_PROPS_TEMPLATE': {
				const { templateId, targetIds } = command.payload;
				const template = propTemplates.get(templateId);
				if (!template) {
					console.error(`Props template ${templateId} not found.`);
					break;
				}
				for (const id of targetIds) {
					const node = getMutableNode(id);
					if (node) {
						Object.assign(node.props, template.props);
						if (template.namedChildren) {
							node.namedChildren = template.namedChildren;
						}
					}
				}
				break;
			}
			case 'REPLACE_CHILDREN': {
				const { parentId, childrenIds } = command.payload;
				const parentNode = getMutableNode(parentId as number);
				if (parentNode) {
					parentNode.children = childrenIds;
				}
				break;
			}
			case 'CREATE_INSTANCE': {
				const { id, type, props, children, namedChildren } = command.payload;
				tempTree.set(id, {
					id,
					type,
					props,
					children: children ?? [],
					namedChildren: namedChildren ?? {}
				});
				break;
			}
			case 'UPDATE_PROPS': {
				const { id, props, namedChildren } = command.payload;
				const node = getMutableNode(id);
				if (node) {
					Object.assign(node.props, props);
					if (namedChildren) {
						node.namedChildren = namedChildren;
					}
				}
				break;
			}
			case 'APPEND_CHILD': {
				const { parentId, childId } = command.payload;
				if (parentId === 'root') {
					tempState.rootNodeId = childId;
				} else {
					const parentNode = getMutableNode(parentId as number);
					if (parentNode) {
						const existingIdx = parentNode.children.indexOf(childId);
						if (existingIdx > -1) parentNode.children.splice(existingIdx, 1);
						parentNode.children.push(childId);
					}
				}
				break;
			}
			case 'REMOVE_CHILD': {
				const { parentId, childId } = command.payload;
				const parentNode = getMutableNode(parentId as number);
				if (parentNode) {
					const index = parentNode.children.indexOf(childId);
					if (index > -1) parentNode.children.splice(index, 1);
				}
				break;
			}
			case 'INSERT_BEFORE': {
				const { parentId, childId, beforeId } = command.payload;
				const parentNode = getMutableNode(parentId as number);
				if (parentNode) {
					const oldIndex = parentNode.children.indexOf(childId);
					if (oldIndex > -1) parentNode.children.splice(oldIndex, 1);
					const insertIndex = parentNode.children.indexOf(beforeId);
					if (insertIndex > -1) {
						parentNode.children.splice(insertIndex, 0, childId);
					} else {
						parentNode.children.push(childId);
					}
				}
				break;
			}
			case 'SHOW_TOAST': {
				const toast = command.payload as unknown as Toast;
				toasts.set(toast.id, toast);
				break;
			}
			case 'UPDATE_TOAST': {
				const { id, ...rest } = command.payload;
				const existingToast = toasts.get(id);
				if (existingToast) {
					toasts.set(id, { ...existingToast, ...rest });
				}
				break;
			}
			case 'HIDE_TOAST': {
				toasts.delete(command.payload.id);
				break;
			}
			case 'CREATE_TEXT_INSTANCE':
				break;
			case 'UPDATE_TEXT':
				break;
			case 'CLEAR_CONTAINER':
				break;
			default:
				console.warn('Unknown command type in ui.store:', (command as Command).type);
		}
	}

	return {
		get uiTree() {
			return uiTree;
		},
		get rootNodeId() {
			return rootNodeId;
		},
		get selectedNodeId() {
			return selectedNodeId;
		},
		set selectedNodeId(id: number | undefined) {
			selectedNodeId = id;
		},
		get pluginList() {
			return pluginList;
		},
		get currentPreferences() {
			return currentPreferences;
		},
		get currentRunningPlugin() {
			return currentRunningPlugin;
		},
		get toasts() {
			return toasts;
		},
		get primaryAction() {
			return actionInfo.primary;
		},
		get secondaryAction() {
			return actionInfo.secondary;
		},
		get actionPanel() {
			return actionInfo.panel;
		},
		get allActions() {
			return actionInfo.allActions;
		},
		applyCommands,
		setPluginList,
		setCurrentPreferences,
		setCurrentRunningPlugin(plugin: PluginInfo | null) {
			currentRunningPlugin = plugin;
		},
		resetForNewPlugin
	};
}

export const uiStore = createUiStore();
