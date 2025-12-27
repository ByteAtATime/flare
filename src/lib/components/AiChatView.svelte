<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { tick, onMount } from 'svelte';
	import { Loader2, Send, Stars, MessageSquare, Plus } from '@lucide/svelte';
	import { focusManager } from '$lib/focus.svelte';
	import { viewManager } from '$lib/viewManager.svelte';
	import HeaderInput from './HeaderInput.svelte';
	import MainLayout from './layout/MainLayout.svelte';
	import Header from './layout/Header.svelte';
	import ActionBar from './nodes/shared/ActionBar.svelte';
	import starsSquareIcon from '$lib/assets/stars-square-1616x16@2x.png?inline';
	import SvelteMarked from 'svelte-marked';
	import { Button } from './ui/button';

	type Props = {
		onBack: () => void;
	};

	type Message = {
		role: 'user' | 'assistant';
		content: string;
	};

	type Conversation = {
		id: string;
		title: string;
		createdAt: number;
		updatedAt: number;
		model?: string;
		messages: Message[];
	};

	let { onBack }: Props = $props();

	let currentConversationId = $state<string | null>(null);
	let conversations = $state<Conversation[]>([]);
	let messages = $state<Message[]>([]);
	let prompt = $state('');
	let isGenerating = $state(false);
	let showSidebar = $state(true);
	let searchInputEl: HTMLInputElement | null = $state(null);
	let scrollContainer: HTMLElement | null = $state(null);

	$effect(() => {
		if (focusManager.activeScope === 'main-input') {
			tick().then(() => {
				searchInputEl?.focus();
			});
		}
	});

	// Auto-scroll to bottom when messages change
	$effect(() => {
		if (messages.length > 0) {
			tick().then(() => {
				if (scrollContainer) {
					scrollContainer.scrollTop = scrollContainer.scrollHeight;
				}
			});
		}
	});

	async function handleSubmit() {
		if (!prompt.trim() || isGenerating) return;

		const userMessage = prompt.trim();
		messages = [...messages, { role: 'user', content: userMessage }];
		prompt = '';
		isGenerating = true;

		// Add placeholder for assistant message
		const assistantMessageId = Date.now().toString();
		messages = [...messages, { role: 'assistant', content: '' }];
		const assistantMessageIndex = messages.length - 1;

		try {
			await invoke('ai_ask_stream', {
				requestId: assistantMessageId,
				prompt: userMessage,
				options: {
					model: 'default',
					creativity: 'medium'
				}
			});
		} catch (error) {
			console.error('AI streaming failed:', error);
			messages[assistantMessageIndex].content =
				'Failed to get response from AI. Please check your settings and API key.';
			isGenerating = false;
		}
	}

	async function loadConversations() {
		try {
			conversations = await invoke<Conversation[]>('list_conversations');
		} catch (error) {
			console.error('Failed to load conversations:', error);
		}
	}

	async function loadConversation(id: string) {
		try {
			const conversation = await invoke<Conversation | null>('get_conversation', { id });
			if (conversation) {
				currentConversationId = conversation.id;
				messages = conversation.messages;
			}
		} catch (error) {
			console.error('Failed to load conversation:', error);
		}
	}

	function newChat() {
		messages = [];
		currentConversationId = null;
		searchInputEl?.focus();
	}

	onMount(() => {
		loadConversations();

		if (viewManager.initialAiPrompt) {
			prompt = viewManager.initialAiPrompt;
			viewManager.initialAiPrompt = null;
			handleSubmit();
		}

		const unlistenChunk = listen<{ request_id: string; text: string }>(
			'ai-stream-chunk',
			(event) => {
				const { text } = event.payload;
				messages[messages.length - 1].content += text;
			}
		);

		const unlistenEnd = listen<{ request_id: string; full_text: string }>(
			'ai-stream-end',
			async () => {
				isGenerating = false;
				// Auto-save after response completes
				if (messages.length > 0) {
					await saveConversation();
				}
			}
		);

		return () => {
			unlistenChunk.then((f) => f());
			unlistenEnd.then((f) => f());
		};
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSubmit();
		}
	}

	async function saveConversation() {
		try {
			if (!currentConversationId) {
				// Create new conversation
				const title = messages[0]?.content.slice(0, 50) || 'New Chat';
				const conversation = await invoke<Conversation>('create_conversation', {
					title,
					model: null
				});
				currentConversationId = conversation.id;
			}
			// Update existing conversation
			await invoke('update_conversation', {
				id: currentConversationId,
				title: null,
				messages: messages
			});
		} catch (error) {
			console.error('Failed to save conversation:', error);
		}
	}

	function clearChat() {
		newChat();
	}
</script>

<MainLayout>
	{#snippet header()}
		<Header showBackButton={true} onPopView={onBack}>
			<HeaderInput
				placeholder="Ask anything..."
				bind:value={prompt}
				bind:ref={searchInputEl}
				autofocus
				class="!pl-2.5"
				onkeydown={handleKeydown}
			/>
			{#if isGenerating}
				<div class="mr-4">
					<Loader2 class="text-muted-foreground size-4 animate-spin" />
				</div>
			{/if}
		</Header>
	{/snippet}

	{#snippet content()}
		<div
			bind:this={scrollContainer}
			class="flex grow flex-col gap-6 overflow-y-auto scroll-smooth p-6"
		>
			{#if messages.length === 0}
				<div class="flex h-full flex-col items-center justify-center text-center">
					<div class="bg-primary/10 mb-4 rounded-2xl p-4">
						<Stars class="text-primary size-12" />
					</div>
					<h2 class="text-2xl font-semibold">How can I help you today?</h2>
					<p class="text-muted-foreground mt-2 max-w-sm">
						Ask anything, from coding questions to general knowledge. Press Ctrl+, to configure.
					</p>
				</div>
			{:else}
				{#each messages as message}
					<div
						class="flex flex-col gap-2 {message.role === 'user'
							? 'ml-12 items-end'
							: 'mr-12 items-start'}"
					>
						<div
							class="rounded-2xl px-4 py-3 text-sm leading-relaxed {message.role === 'user'
								? 'bg-primary text-primary-foreground shadow-sm'
								: 'bg-muted/50 border-border/50 border'}"
						>
							{#if message.role === 'assistant'}
								<div class="prose prose-sm prose-invert max-w-none">
									<SvelteMarked source={message.content} />
								</div>
							{:else}
								{message.content}
							{/if}
						</div>
					</div>
				{/each}
			{/if}
		</div>
	{/snippet}

	{#snippet footer()}
		<ActionBar
			actions={[
				{
					title: 'Send',
					shortcut: { key: 'Enter', modifiers: [] },
					handler: handleSubmit
				},
				{
					title: 'New Chat',
					shortcut: { key: 'n', modifiers: ['ctrl'] },
					handler: newChat
				},
				{
					title: 'Configure AI',
					shortcut: { key: ',', modifiers: ['ctrl'] },
					handler: () => viewManager.showSettings()
				},
				{
					title: 'Clear Chat',
					shortcut: { key: 'l', modifiers: ['ctrl'] },
					handler: clearChat
				}
			]}
			icon={starsSquareIcon}
			title="AI Chat"
		/>
	{/snippet}
</MainLayout>

<style>
	:global(.prose pre) {
		background-color: rgba(0, 0, 0, 0.2);
		padding: 1rem;
		border-radius: 0.5rem;
		overflow-x: auto;
	}
	:global(.prose code) {
		font-family:
			ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New',
			monospace;
		font-size: 0.9em;
	}
	:global(.prose p:first-child) {
		margin-top: 0;
	}
	:global(.prose p:last-child) {
		margin-bottom: 0;
	}
</style>
