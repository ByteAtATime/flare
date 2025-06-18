import type { UINode } from '$lib/types';
import type { Command } from '@raycast-linux/protocol';
import type { PluginInfo } from '@raycast-linux/protocol';
import type { KeyboardShortcut } from '$lib/props/actions';
import { SvelteMap } from 'svelte/reactivity';

export type Toast = {
	id: number;
	title: string;
	message?: string;
	style?: 'SUCCESS' | 'FAILURE' | 'ANIMATED';
	primaryAction?: { title: string; onAction: boolean; shortcut?: KeyboardShortcut };
	secondaryAction?: { title: string; onAction: boolean; shortcut?: KeyboardShortcut };
};

function createUiStore() {
	// we're not using SvelteMap here because we're making a lot of mutations to the tree
	// svelte tries to be helpful and rerenders every time we mutate the tree
	// instead, we make a temporary copy of the tree and apply the changes to that
	let uiTree = $state(new Map<number, UINode>());
	let rootNodeId = $state<number | null>(null);
	let selectedNodeId = $state<number | undefined>(undefined);
	let pluginList = $state<PluginInfo[]>([]);
	let currentPreferences = $state<Record<string, unknown>>({});
	const toasts = new SvelteMap<number, Toast>();

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
	};

	function processSingleCommand(
		command: Command,
		tempTree: Map<number, UINode>,
		tempState: { rootNodeId: number | null },
		getMutableNode: (id: number) => UINode | undefined
	) {
		switch (command.type) {
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
		get toasts() {
			return toasts;
		},
		applyCommands,
		setPluginList,
		setCurrentPreferences,
		resetForNewPlugin
	};
}

export const uiStore = createUiStore();
