<script lang="ts">
	import type { UnifiedItem } from '$lib/command-palette.svelte';
	import ActionBar from '$lib/components/nodes/shared/ActionBar.svelte';
	import negativeRaycastLogo from '$lib/assets/raycast-logo-neg-1616x16@2x.png';
	import positiveRaycastLogo from '$lib/assets/raycast-logo-pos-1616x16@2x.png';
	import type { ActionDefinition } from '../nodes/shared/actions';

	type Props = {
		selectedItem: UnifiedItem | undefined;
		actions: {
			handleEnter: () => Promise<void>;
			handleResetRanking: () => Promise<void>;
			handleCopyDeeplink: () => void;
			handleConfigureCommand: () => void;
			handleCopyAppName: () => void;
			handleCopyAppPath: () => void;
			handleHideApp: () => Promise<void>;
		};
		setSearchText: (text: string) => void;
	};

	let { selectedItem, actions: barActions, setSearchText }: Props = $props();

	const actions: ActionDefinition[] = $derived.by(() => {
		if (!selectedItem) return [];

		if (selectedItem.type === 'calculator') {
			return [
				{
					title: 'Copy Answer',
					handler: barActions.handleEnter
				},
				{
					title: 'Put Answer in Search Bar',
					shortcut: { key: 'enter', modifiers: ['ctrl', 'shift'] },
					handler: () => setSearchText(selectedItem.data.result)
				}
			];
		}

		if (selectedItem.type === 'plugin') {
			return [
				{
					title: 'Open Command',
					handler: barActions.handleEnter
				},
				{
					title: 'Reset Ranking',
					handler: barActions.handleResetRanking
				},
				{
					title: 'Copy Deeplink',
					shortcut: { key: 'c', modifiers: ['ctrl', 'shift'] },
					handler: barActions.handleCopyDeeplink
				},
				{
					title: 'Configure Command',
					shortcut: { key: ',', modifiers: ['ctrl', 'shift'] },
					handler: barActions.handleConfigureCommand
				}
			];
		}

		if (selectedItem.type === 'app') {
			return [
				{
					title: 'Open Application',
					handler: barActions.handleEnter
				},
				{
					title: 'Reset Ranking',
					handler: barActions.handleResetRanking
				},
				{
					title: 'Copy Name',
					shortcut: { key: '.', modifiers: ['ctrl'] },
					handler: barActions.handleCopyAppName
				},
				{
					title: 'Copy Path',
					shortcut: { key: '.', modifiers: ['ctrl', 'shift'] },
					handler: barActions.handleCopyAppPath
				},
				{
					title: 'Hide Application',
					shortcut: { key: 'h', modifiers: ['ctrl'] },
					handler: barActions.handleHideApp
				}
			];
		}

		if (selectedItem.type === 'quicklink') {
			return [
				{
					title: 'Open Quicklink',
					handler: barActions.handleEnter
				}
			];
		}

		return [];
	});
</script>

{#if selectedItem}
	<ActionBar {actions}>
		{#snippet title()}
			<div class="pl-1">
				<img src={positiveRaycastLogo} alt="Flare" class="size-5 brightness-50 dark:hidden" />
				<img src={negativeRaycastLogo} alt="Flare" class="hidden size-5 brightness-50 dark:block" />
			</div>
		{/snippet}
	</ActionBar>
{/if}
