<script lang="ts">
	import { ArrowLeft, Check } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import Icon from './Icon.svelte';
	import type { ImageLike } from '@flare/protocol';

	type Props = {
		providerName: string;
		providerIcon?: ImageLike;
		description?: string;
		authUrl: string;
		status: 'initial' | 'authorizing' | 'success' | 'error';
		onSignIn: () => void;
		onBack: () => void;
	};

	let { providerName, providerIcon, description, authUrl, status, onSignIn, onBack }: Props =
		$props();

	let isLinkCopied = $state(false);

	function handleCopyLink() {
		writeText(authUrl);
		isLinkCopied = true;
		setTimeout(() => {
			isLinkCopied = false;
		}, 2000);
	}
</script>

<div class="flex h-screen flex-col items-center justify-center">
	<header class="absolute top-4 left-4">
		<Button variant="ghost" size="icon" class="rounded-full text-white/80" onclick={onBack}>
			<ArrowLeft class="size-5" />
		</Button>
	</header>

	<div class="flex flex-col items-center gap-4 text-center">
		<div class="relative mb-2 flex h-20 w-20 items-center justify-center">
			<div
				class="bg-background absolute z-10 flex size-12 items-center justify-center rounded-2xl border border-white/10"
			>
				<Icon icon="raycast-logo-neg-16" class="size-7" />
				{#if status === 'success'}
					<div
						class="border-background absolute -top-2 -right-2 flex size-6 items-center justify-center rounded-full border-2 bg-green-500 text-white"
					>
						<Check class="size-4" />
					</div>
				{/if}
			</div>
			{#if providerIcon}
				<div class="absolute -right-3 bottom-1 z-0">
					<!-- <Icon icon={providerIcon} class="size-10" /> -->
				</div>
			{/if}
		</div>

		<h1 class="text-4xl font-bold text-white">{providerName}</h1>

		{#if status === 'success'}
			<p class="text-lg text-white/70">Successfully connected to {providerName}</p>
		{:else}
			<p class="text-lg text-white/70">{description}</p>
		{/if}

		{#if status === 'initial'}
			<Button
				class="mt-4 bg-white/10 px-8 py-3 text-base font-semibold text-white hover:bg-white/20"
				onclick={onSignIn}>Sign in with {providerName}</Button
			>
		{/if}
	</div>

	{#if status === 'initial'}
		<footer class="absolute bottom-8">
			<span class="text-sm text-white/50">
				Need to open in another browser?
				<button class="font-medium text-white/80 hover:underline" onclick={handleCopyLink}>
					{#if isLinkCopied}
						Copied!
					{:else}
						Copy authorization link
					{/if}
				</button>
			</span>
		</footer>
	{/if}
</div>
