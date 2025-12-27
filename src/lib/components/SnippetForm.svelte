<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import Icon from '$lib/components/Icon.svelte';
	import { Save } from '@lucide/svelte';
	import { uiStore } from '$lib/ui.svelte';
	import MainLayout from './layout/MainLayout.svelte';
	import Header from './layout/Header.svelte';
	import ActionBar from './nodes/shared/ActionBar.svelte';
	import snippetIcon from '$lib/assets/snippets-package-1616x16@2x.png?inline';

	type Snippet = {
		id: number;
		name: string;
		keyword: string;
		content: string;
		createdAt: string;
		updatedAt: string;
		timesUsed: number;
		lastUsedAt: string;
	};

	type Props = {
		onBack: () => void;
		onSave: () => void;
		editSnippet?: Snippet;
	};

	let { onBack, onSave, editSnippet }: Props = $props();

	const isEditing = $derived(!!editSnippet);

	let name = $state(editSnippet?.name ?? '');
	let keyword = $state(editSnippet?.keyword ?? '');
	let snippetContent = $state(editSnippet?.content ?? '');
	let error = $state('');

	type ParsedPart = {
		text: string;
		type: 'text' | 'valid-bracket' | 'invalid-bracket' | 'valid-name' | 'invalid-name';
	};

	const PLACEHOLDER_REGEX =
		/\{(?<name>\w+)(?<attributes>(?:\s+\w+=(?:"[^"]*"|\S+))*)?(?<modifiers>(?:\s*\|\s*[\w%-]+)*)\}/g;

	const VALID_PLACEHOLDERS = new Set([
		'clipboard',
		'snippet',
		'cursor',
		'date',
		'time',
		'datetime',
		'day',
		'uuid'
	]);
	const VALID_MODIFIERS = new Set([
		'uppercase',
		'lowercase',
		'trim',
		'percent-encode',
		'json-stringify'
	]);
	const VALID_ATTRIBUTES: Record<string, Set<string>> = {
		clipboard: new Set(['offset']),
		snippet: new Set(['name']),
		date: new Set(['offset', 'format']),
		time: new Set(['offset', 'format']),
		datetime: new Set(['offset', 'format']),
		day: new Set(['offset', 'format'])
	};
	const ATTRIBUTE_REGEX = /\s*(?<key>\w+)\s*=\s*"(?:[^"]*)"/g;

	function parseAttributes(attrStr: string | undefined): Record<string, string> {
		if (!attrStr) return {};
		const attributes: Record<string, string> = {};
		for (const match of attrStr.matchAll(ATTRIBUTE_REGEX)) {
			if (match.groups) {
				attributes[match.groups.key] = 'dummy';
			}
		}
		return attributes;
	}

	function parseModifiers(modStr: string | undefined): string[] {
		if (!modStr) return [];
		return modStr
			.split('|')
			.map((s) => s.trim())
			.filter(Boolean);
	}

	function validatePlaceholder(
		name: string,
		attributes: Record<string, string>,
		modifiers: string[]
	): boolean {
		if (!VALID_PLACEHOLDERS.has(name)) return false;

		const validAttrsForPlaceholder = VALID_ATTRIBUTES[name] || new Set();
		for (const attrName in attributes) {
			if (!validAttrsForPlaceholder.has(attrName)) return false;
		}

		for (const mod of modifiers) {
			if (!VALID_MODIFIERS.has(mod)) return false;
		}

		return true;
	}

	const parsedContent = $derived.by(() => {
		if (!snippetContent) return [];

		const parts: ParsedPart[] = [];
		let lastIndex = 0;
		let cursorCount = 0;

		for (const match of snippetContent.matchAll(PLACEHOLDER_REGEX)) {
			if (match.index! > lastIndex) {
				parts.push({
					text: snippetContent.substring(lastIndex, match.index),
					type: 'text'
				});
			}

			const placeholderName = match.groups!.name;
			const attributes = parseAttributes(match.groups!.attributes);
			const modifiers = parseModifiers(match.groups!.modifiers);

			let isValid = validatePlaceholder(placeholderName, attributes, modifiers);

			if (placeholderName === 'cursor') {
				cursorCount++;
				if (cursorCount > 1) {
					isValid = false;
				}
			}

			const bracketType = isValid ? 'valid-bracket' : 'invalid-bracket';
			const nameType = isValid ? 'valid-name' : 'invalid-name';

			parts.push({ text: '{', type: bracketType });
			parts.push({
				text: match[0].slice(1, -1),
				type: nameType
			});
			parts.push({ text: '}', type: bracketType });

			lastIndex = match.index! + match[0].length;
		}

		if (lastIndex < snippetContent.length) {
			parts.push({ text: snippetContent.substring(lastIndex), type: 'text' });
		}

		return parts;
	});

	async function handleSave() {
		if (!name.trim() || !keyword.trim() || !snippetContent.trim()) {
			error = 'All fields are required.';
			return;
		}
		error = '';

		try {
			if (isEditing && editSnippet) {
				await invoke('update_snippet', {
					id: editSnippet.id,
					name,
					keyword,
					content: snippetContent
				});
				uiStore.toasts.set(Date.now(), {
					id: Date.now(),
					title: 'Snippet Updated',
					style: 'SUCCESS'
				});
			} else {
				await invoke('create_snippet', { name, keyword, content: snippetContent });
				uiStore.toasts.set(Date.now(), {
					id: Date.now(),
					title: 'Snippet Created',
					style: 'SUCCESS'
				});
			}
			onSave();
		} catch (e) {
			const errorMessage = e instanceof Error ? e.message : String(e);
			error = errorMessage;
			console.error('Failed to save snippet:', e);
		}
	}
</script>

<MainLayout>
	{#snippet header()}
		<Header showBackButton={true} onPopView={onBack}>
			<div class="flex items-center gap-3 !pl-2.5">
				<Icon icon="snippets-16" class="size-6" />
				<h1 class="text-lg font-medium">{isEditing ? 'Edit Snippet' : 'Create Snippet'}</h1>
			</div>
		</Header>
	{/snippet}
	{#snippet content()}
		<div class="grow overflow-y-auto p-6">
			<div class="mx-auto max-w-xl space-y-6">
				<div class="grid grid-cols-[120px_1fr] items-center gap-4">
					<label for="name" class="text-right text-sm text-gray-400">Name</label>
					<Input id="name" placeholder="Snippet name" bind:value={name} />
				</div>
				<div class="grid grid-cols-[120px_1fr] items-center gap-4">
					<label for="keyword" class="text-right text-sm text-gray-400">Keyword</label>
					<Input id="keyword" placeholder="!email" bind:value={keyword} />
				</div>

				<div class="grid grid-cols-[120px_1fr] items-start gap-4">
					<label for="content" class="pt-2 text-right text-sm text-gray-400">Snippet</label>
					<div class="grid w-full">
						<div
							aria-hidden="true"
							class="pointer-events-none col-start-1 row-start-1 min-h-32 w-full rounded-md border-transparent bg-transparent px-3 py-2 font-mono text-sm break-words whitespace-pre-wrap"
						>
							{#each parsedContent as part}
								<span
									class:text-blue-400={part.type === 'valid-bracket'}
									class:text-red-400={part.type === 'invalid-bracket' ||
										part.type === 'invalid-name'}
									class:text-foreground={part.type === 'text' || part.type === 'valid-name'}
								>
									{part.text}
								</span>
							{/each}
							<span>​</span>
						</div>
						<Textarea
							id="content"
							placeholder="Enter your snippet content... e.g. Hello {'{'}clipboard | uppercase}!"
							bind:value={snippetContent}
							class="caret-foreground col-start-1 row-start-1 min-h-32 resize-none !bg-transparent font-mono text-transparent"
							spellcheck={false}
						/>
					</div>
				</div>

				{#if error}
					<p class="text-center text-red-500">{error}</p>
				{/if}
			</div>
		</div>
	{/snippet}
	{#snippet footer()}
		<ActionBar
			actions={[
				{
					title: isEditing ? 'Save Snippet' : 'Create Snippet',
					handler: handleSave,
					shortcut: { key: 'enter', modifiers: ['cmd'] }
				}
			]}
			icon={snippetIcon}
			title={isEditing ? 'Edit Snippet' : 'Create Snippet'}
		>
			{#snippet primaryAction({ props })}
				<Button {...props} onclick={handleSave}
					><Save class="mr-2 size-4" /> {isEditing ? 'Save' : 'Create'} Snippet</Button
				>
			{/snippet}
		</ActionBar>
	{/snippet}
</MainLayout>
