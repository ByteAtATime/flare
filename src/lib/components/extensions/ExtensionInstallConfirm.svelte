<script lang="ts">
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import { Button } from '$lib/components/ui/button';
	import * as Accordion from '$lib/components/ui/accordion';
	import { TriangleAlert } from '@lucide/svelte';

	type Violation = {
		commandName: string;
		commandTitle?: string;
		reason: string;
	};

	type Props = {
		violations: Violation[];
		open: boolean;
		onconfirm: () => void;
		oncancel: () => void;
	};

	let { violations, open = $bindable(), onconfirm, oncancel }: Props = $props();

	const isTruncated = $derived(violations.length > 3);
	const truncatedViolations = $derived(violations.slice(0, 3));
</script>

<AlertDialog.Root bind:open onOpenChange={(isOpen) => !isOpen && oncancel()}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<div class="flex flex-col items-center gap-2 text-center">
				<TriangleAlert class="size-12 text-yellow-400" />
				<AlertDialog.Title>Potential Incompatibility Detected</AlertDialog.Title>
			</div>
			<AlertDialog.Description class="text-center">
				This extension may not work as expected on your system. We recommend proceeding with
				caution.
				<Accordion.Root class="w-full pt-4" type="multiple">
					<Accordion.Item value="details">
						<Accordion.Trigger>Technical Details</Accordion.Trigger>
						<Accordion.Content>
							<ul class="list-disc space-y-2 pl-5 text-left text-xs">
								{#each truncatedViolations as violation}
									<li>
										<strong>{violation.commandTitle ?? violation.commandName}</strong>
										{#if violation.commandTitle && violation.commandTitle !== violation.commandName}
											<span class="text-muted-foreground ml-1">({violation.commandName})</span>
										{/if}
										: {violation.reason}
									</li>
								{/each}
								{#if isTruncated}
									<li>... {violations.length - 3} more warnings</li>
								{/if}
							</ul>
						</Accordion.Content>
					</Accordion.Item>
				</Accordion.Root>
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel onclick={() => oncancel()}>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action>
				{#snippet child({ props })}
					<Button {...props} onclick={() => onconfirm()}>Install anyway</Button>
				{/snippet}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
