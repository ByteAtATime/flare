<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Pencil, Plus, Trash2 } from '@lucide/svelte';
	import { aiStore } from '$lib/ai.svelte';
	import * as Dialog from '$lib/components/ui/dialog';

	let isDialogOpen = $state(false);
	let editingId = $state<string | null>(null);
	let name = $state('');
	let template = $state('');
	let icon = $state('');

	function openCreate() {
		editingId = null;
		name = '';
		template = '';
		icon = '';
		isDialogOpen = true;
	}

	function openEdit(preset: any) {
		editingId = preset.id;
		name = preset.name;
		template = preset.template;
		icon = preset.icon || '';
		isDialogOpen = true;
	}

	async function handleDelete(id: string) {
		if (confirm('Are you sure you want to delete this preset?')) {
			await aiStore.deletePreset(id);
		}
	}

	async function handleSave() {
		if (!name.trim() || !template.trim()) return;

		try {
			if (editingId) {
				await aiStore.updatePreset(editingId, name, template, icon || null);
			} else {
				await aiStore.createPreset(name, template, icon || null);
			}
			isDialogOpen = false;
		} catch (error) {
			console.error('Failed to save preset:', error);
		}
	}
</script>

<div class="space-y-4">
	<div class="flex items-center justify-between">
		<h3 class="text-lg font-medium">AI Presets</h3>
		<Button size="sm" onclick={openCreate}>
			<Plus class="mr-2 h-4 w-4" />
			Add Preset
		</Button>
	</div>

	<div class="grid gap-4">
		{#each aiStore.presets as preset}
			<div class="border-border flex items-center justify-between rounded-lg border p-4">
				<div>
					<div class="font-medium">{preset.name}</div>
					<div class="text-muted-foreground line-clamp-1 text-sm">{preset.template}</div>
				</div>
				<div class="flex gap-2">
					<Button variant="ghost" size="icon" onclick={() => openEdit(preset)}>
						<Pencil class="h-4 w-4" />
					</Button>
					<Button variant="ghost" size="icon" onclick={() => handleDelete(preset.id)}>
						<Trash2 class="h-4 w-4 text-red-500" />
					</Button>
				</div>
			</div>
		{/each}
	</div>

	<Dialog.Root bind:open={isDialogOpen}>
		<Dialog.Content class="sm:max-w-[425px]">
			<Dialog.Header>
				<Dialog.Title>{editingId ? 'Edit Preset' : 'Create Preset'}</Dialog.Title>
				<Dialog.Description>
					Create a new AI command preset. use <code>{'{selection}'}</code> to insert selected text.
				</Dialog.Description>
			</Dialog.Header>
			<div class="grid gap-4 py-4">
				<div class="grid gap-2">
					<label for="name" class="text-sm font-medium">Name</label>
					<Input id="name" bind:value={name} placeholder="e.g. Summarize" />
				</div>
				<div class="grid gap-2">
					<label for="template" class="text-sm font-medium">Prompt Template</label>
					<Textarea
						id="template"
						bind:value={template}
						placeholder={'Summarize the following text: {selection}'}
						class="min-h-[100px]"
					/>
				</div>
				<div class="grid gap-2">
					<label for="icon" class="text-sm font-medium">Icon (optional)</label>
					<Input id="icon" bind:value={icon} placeholder="e.g. pencil-1" />
				</div>
			</div>
			<Dialog.Footer>
				<Button onclick={handleSave}>Save changes</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
</div>
