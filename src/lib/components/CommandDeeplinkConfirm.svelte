<script lang="ts">
	import type { PluginInfo } from '@raycast-linux/protocol';
	import { Button } from './ui/button';
	import Icon from './Icon.svelte';
	import path from 'path';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';

	type Props = {
		plugin: PluginInfo;
		open?: boolean;
		onconfirm: () => void;
		oncancel: () => void;
	};
	let { plugin, open = $bindable(true), onconfirm, oncancel }: Props = $props();

	const assetsPath = $derived(path.dirname(plugin.pluginPath) + '/assets');
</script>

<AlertDialog.Root bind:open onOpenChange={(isOpen) => !isOpen && oncancel()}>
	<AlertDialog.Content class="w-fit">
		<AlertDialog.Header class="items-center text-center">
			<Icon icon={plugin.icon} class="size-16" {assetsPath} />
			<AlertDialog.Title class="text-xl font-semibold">
				Request to open {plugin.title}
			</AlertDialog.Title>
			<AlertDialog.Description class="text-center text-sm">
				The command was triggered from outside of Flare. If you did not do this, please cancel the
				operation.
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer class="mt-2 !flex-col gap-2">
			<AlertDialog.Action>
				{#snippet child({ props })}
					<Button {...props} onclick={onconfirm} class="w-full text-base" size="lg">
						Open Command
					</Button>
				{/snippet}
			</AlertDialog.Action>
			<!-- TODO: implement "always open" -->
			<Button onclick={onconfirm} variant="secondary" class="w-full text-base" size="lg">
				Always Open Command
			</Button>
			<AlertDialog.Cancel>
				{#snippet child({ props })}
					<Button {...props} variant="ghost" class="w-full text-base" size="lg">Cancel</Button>
				{/snippet}
			</AlertDialog.Cancel>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
