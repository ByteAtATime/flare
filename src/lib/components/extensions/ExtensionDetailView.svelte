<script lang="ts">
	import type { Extension, Command as ExtensionCommand } from '$lib/store';
	import { Button } from '$lib/components/ui/button';
	import Icon from '../Icon.svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { Separator } from '../ui/separator';
	import * as Carousel from '$lib/components/ui/carousel/index.js';
	import ActionBar from '$lib/components/nodes/shared/ActionBar.svelte';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import * as Command from '$lib/components/ui/command/index.js';
	import aiIcon from '$lib/assets/stars-square-1616x16@2x.png';
	import KeyboardShortcut from '../KeyboardShortcut.svelte';
	import { uiStore } from '$lib/ui.svelte';
	import { viewManager } from '$lib/viewManager.svelte';
	import CompatibilityBadge from './CompatibilityBadge.svelte';
	import { invoke } from '@tauri-apps/api/core';

	type Props = {
		extension: Extension;
		isInstalling: boolean;
		onInstall: () => void;
		onOpenLightbox: (imageUrl: string) => void;
	};

	let { extension, isInstalling, onInstall, onOpenLightbox }: Props = $props();

	let openCommandsPopover = $state(false);
	let isUninstalling = $state(false);

	function formatTimeAgo(timestamp: number) {
		const date = new Date(timestamp * 1000);
		const now = new Date();
		const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);
		let interval = seconds / 31536000;
		if (interval > 1) {
			const years = Math.floor(interval);
			return `${years} year${years > 1 ? 's' : ''} ago`;
		}
		interval = seconds / 2592000;
		if (interval > 1) {
			const months = Math.floor(interval);
			return `${months} month${months > 1 ? 's' : ''} ago`;
		}
		interval = seconds / 604800;
		if (interval > 1) {
			const weeks = Math.floor(interval);
			return `${weeks} week${weeks > 1 ? 's' : ''} ago`;
		}
		interval = seconds / 86400;
		if (interval > 1) {
			const days = Math.floor(interval);
			return `${days} day${days > 1 ? 's' : ''} ago`;
		}
		interval = seconds / 3600;
		if (interval > 1) {
			const hours = Math.floor(interval);
			return `${hours} hour${hours > 1 ? 's' : ''} ago`;
		}
		interval = seconds / 60;
		if (interval > 1) {
			const minutes = Math.floor(interval);
			return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
		}
		return `${Math.floor(seconds)} second${seconds !== 1 ? 's' : ''} ago`;
	}

	const isInstalled = $derived(uiStore.pluginList.some((p) => p.pluginName === extension.name));

	const installedCommandsInfo = $derived(
		isInstalled ? uiStore.pluginList.filter((p) => p.pluginName === extension.name) : []
	);

	const compatibilityInfo = $derived.by(() => {
		if (!isInstalled || installedCommandsInfo.length === 0) {
			return null;
		}

		// Calculate average compatibility score from all commands
		const scores = installedCommandsInfo
			.map((p) => p.compatibilityScore)
			.filter((score): score is number => score !== undefined && score !== null);

		if (scores.length === 0) return null;

		const avgScore = Math.round(scores.reduce((a, b) => a + b, 0) / scores.length);

		// Collect all unique warnings
		const allWarnings = installedCommandsInfo
			.flatMap((p) => p.compatibilityWarnings || [])
			.filter(
				(w, i, arr) =>
					arr.findIndex((v) => v.commandName === w.commandName && v.reason === w.reason) === i
			);

		return {
			score: avgScore,
			warnings: allWarnings
		};
	});

	const screenshots = $derived.by(() => {
		if (extension.metadata && extension.metadata.length > 0) {
			return extension.metadata;
		}
		if (extension.metadata_count > 0) {
			return Array.from(
				{ length: extension.metadata_count },
				(_, i) => `${extension.readme_assets_path}metadata/${extension.name}-${i + 1}.png`
			);
		}
		return [];
	});

	function handleOpenCommand(command: ExtensionCommand) {
		const pluginInfo = installedCommandsInfo.find((p) => p.commandName === command.name);
		if (pluginInfo) {
			viewManager.runPlugin(pluginInfo);
		} else {
			console.error('Could not find installed plugin info for command', command);
		}
	}

	async function handleUninstall() {
		if (isUninstalling) return;
		// Use the pluginName from the installed plugin info, not extension.name from the store API
		const installedPlugin = installedCommandsInfo[0];
		if (!installedPlugin) {
			console.error('No installed plugin info found');
			return;
		}
		isUninstalling = true;
		try {
			await invoke('uninstall_extension', { slug: installedPlugin.pluginName });
			onInstall(); // Refresh plugin list
		} catch (e) {
			console.error('Uninstall failed', e);
		} finally {
			isUninstalling = false;
		}
	}

	const actions = $derived.by(() => {
		if (isInstalled)
			return [
				{ title: 'Show Commands', handler: () => {} },
				{
					title: isUninstalling ? 'Uninstalling...' : 'Uninstall Extension',
					handler: handleUninstall,
					disabled: isUninstalling
				}
			];

		return [
			{
				title: isInstalling ? 'Installing...' : 'Install Extension',
				handler: onInstall,
				disabled: isInstalling
			}
		];
	});
</script>

<div class="flex grow flex-col gap-6 overflow-x-hidden overflow-y-auto p-6">
	<div class="flex items-center gap-6">
		<Icon
			icon={extension.icons.light
				? { source: extension.icons.light, mask: 'roundedRectangle' }
				: undefined}
			class="size-16"
		/>
		<div>
			<h1 class="text-lg font-bold">{extension.title}</h1>
			<div class="mt-2 flex items-center gap-2">
				<div class="flex items-center gap-1 text-sm">
					<Icon
						icon={extension.author.avatar
							? { source: extension.author.avatar, mask: 'circle' }
							: undefined}
						class="size-[18px]"
					/>
					<span>{extension.author.name}</span>
				</div>
				<Separator orientation="vertical" class="!h-4" />
				<div class="flex items-center gap-1 text-sm">
					<Icon icon="arrow-down-circle-16" class="text-muted-foreground fill-none" />
					<span>{extension.download_count.toLocaleString()} Installs</span>
				</div>
				{#if extension.categories?.includes('AI Extensions')}
					<Separator orientation="vertical" class="!h-4" />
					<div class="text-muted-foreground flex items-center gap-1 text-sm">
						<div
							class="size-4"
							style="mask: url({aiIcon}) no-repeat center; mask-size: contain; background-color: currentColor;"
						></div>

						<span>AI Extension</span>
					</div>
				{/if}
			</div>
		</div>
		{#if isInstalled}
			<div class="ml-auto flex items-center rounded bg-[#4EF8A7]/15 px-2 text-[#4EF8A7]">
				<Icon
					icon={{ source: 'check-circle-16', tintColor: 'raycast-green' }}
					class="mr-1 size-[18px]"
				/>

				Installed
			</div>
		{/if}
	</div>

	<Separator />

	{#if screenshots.length > 0}
		<Carousel.Root>
			<Carousel.Content>
				{#each screenshots as imageUrl, i (imageUrl)}
					<Carousel.Item class="grow-0 basis-auto">
						<button class="w-full cursor-pointer" onclick={() => onOpenLightbox(imageUrl)}>
							<img
								src={imageUrl}
								alt={`Screenshot ${i + 1} for ${extension.title}`}
								class="h-[140px] rounded-lg bg-white/5 object-cover"
								loading="lazy"
							/>
						</button>
					</Carousel.Item>
				{/each}
			</Carousel.Content>
			<Carousel.Previous class="-left-4" variant="default" />
			<Carousel.Next class="-right-4" variant="default" />
		</Carousel.Root>
	{/if}

	<Separator class="-mx-6 !w-auto" />

	<div class="grid grid-cols-[2fr_auto_1fr] gap-x-4">
		<div class="flex flex-col gap-4">
			<div>
				<h2 class="text-muted-foreground mb-1 text-xs font-medium uppercase">Description</h2>
				<p>{extension.description}</p>
			</div>

			<Separator />

			<div>
				<h2 class="text-muted-foreground mb-2 text-xs font-medium uppercase">Commands</h2>
				<div class="flex flex-col gap-4">
					{#each extension.commands as command (command.id)}
						{@const commandIcon = command.icons.light
							? { source: command.icons.light, mask: 'roundedRectangle' as const }
							: undefined}
						{@const extensionIcon = extension.icons.light
							? { source: extension.icons.light, mask: 'roundedRectangle' as const }
							: undefined}
						<div class="flex items-start gap-3">
							<div>
								<div class="mb-1 flex items-center gap-2 text-sm font-medium">
									<Icon icon={commandIcon ?? extensionIcon ?? undefined} class="size-[22px]" />
									<span>{command.title}</span>
								</div>
								<p class="text-muted-foreground text-xs">{command.description}</p>
							</div>
						</div>
					{/each}
				</div>
			</div>
		</div>

		<Separator orientation="vertical" class="-mt-6" />

		<div class="space-y-8">
			{#if extension.readme_url}
				<div>
					<h2 class="text-muted-foreground mb-1 text-xs font-medium uppercase">README</h2>
					<Button
						variant="link"
						class="text-foreground group w-full justify-between !p-0"
						onclick={() => openUrl(extension.readme_url!)}
					>
						Open README
						<Icon
							icon="arrow-ne-16"
							class="text-muted-foreground group-hover:text-foreground size-4"
						/>
					</Button>
				</div>
			{/if}
			<div>
				<h3 class="text-muted-foreground mb-1 text-xs font-medium uppercase">Last updated</h3>
				<p>{formatTimeAgo(extension.updated_at)}</p>
			</div>
			<div>
				<h3 class="text-muted-foreground mb-1 text-xs font-medium uppercase">Contributors</h3>
				<div class="flex flex-wrap gap-2">
					{#each extension.contributors as contributor (contributor.handle)}
						<a
							href="https://github.com/{contributor.github_handle}"
							target="_blank"
							class="flex items-center gap-2"
							rel="noopener noreferrer"
						>
							<Icon
								icon={contributor.avatar
									? { source: contributor.avatar, mask: 'circle' }
									: undefined}
								class="size-6"
							/>
						</a>
					{/each}
				</div>
			</div>
			{#if extension.categories?.length > 0}
				<div>
					<h3 class="text-muted-foreground mb-1 text-xs font-medium uppercase">Categories</h3>
					<div class="flex flex-wrap gap-1.5">
						{#each extension.categories as category (category)}
							<span
								class="rounded-full bg-blue-900/50 px-2 py-0.5 text-xs font-semibold text-blue-300"
							>
								{category}
							</span>
						{/each}
					</div>
				</div>
			{/if}
			{#if isInstalled && compatibilityInfo}
				<div>
					<h3 class="text-muted-foreground mb-2 text-xs font-medium uppercase">
						Linux Compatibility
					</h3>
					<div class="space-y-3">
						<CompatibilityBadge score={compatibilityInfo.score} size="lg" showLabel={true} />

						{#if compatibilityInfo.warnings.length > 0}
							<div class="mt-3 space-y-2">
								<p class="text-muted-foreground text-xs font-medium">Detected Issues:</p>
								{#each compatibilityInfo.warnings as warning (warning.commandName + warning.reason)}
									<div class="rounded-md bg-yellow-900/20 p-2 text-xs">
										{#if warning.commandTitle}
											<span class="font-semibold">{warning.commandTitle}:</span>
										{/if}
										<span class="text-yellow-200">{warning.reason}</span>
									</div>
								{/each}
							</div>
						{:else}
							<p class="text-muted-foreground text-xs">No compatibility issues detected.</p>
						{/if}
					</div>
				</div>
			{/if}

			{#if extension.source_url}
				<div>
					<h3 class="text-muted-foreground mb-1 text-xs font-medium uppercase">Source Code</h3>
					<Button
						variant="link"
						class="text-foreground group w-full justify-between !p-0"
						onclick={() => openUrl(extension.source_url)}
					>
						View Code
						<Icon
							icon="arrow-ne-16"
							class="text-muted-foreground group-hover:text-foreground size-4"
						/>
					</Button>
				</div>
			{/if}
		</div>
	</div>
</div>

<ActionBar
	title={extension.title}
	icon={extension.icons.light
		? { source: extension.icons.light, mask: 'roundedRectangle' }
		: undefined}
>
	{#snippet primaryAction({ props })}
		{#if isInstalled}
			<Button
				variant="ghost"
				size="action"
				onclick={handleUninstall}
				disabled={isUninstalling}
				class="text-destructive"
			>
				{isUninstalling ? 'Uninstalling...' : 'Uninstall'}
				<KeyboardShortcut shortcut={{ key: 'backspace', modifiers: ['ctrl'] }} />
			</Button>
			<Popover.Root bind:open={openCommandsPopover}>
				<Popover.Trigger>
					{#snippet child({ props: triggerProps })}
						<Button {...triggerProps} {...props}>
							Open Commands...
							<KeyboardShortcut shortcut={{ key: 'enter', modifiers: [] }} />
						</Button>
					{/snippet}
				</Popover.Trigger>
				<Popover.Content class="w-80 p-0" side="top" align="start">
					<Command.Root>
						<Command.Input placeholder="Search commands..." />
						<Command.Empty>No results.</Command.Empty>
						<Command.List>
							{#each extension.commands as command (command.id)}
								{@const commandIcon = command.icons.light
									? { source: command.icons.light, mask: 'roundedRectangle' as const }
									: undefined}
								{@const extensionIcon = extension.icons.light
									? { source: extension.icons.light, mask: 'roundedRectangle' as const }
									: undefined}
								<Command.Item
									value={command.title}
									onSelect={() => {
										handleOpenCommand(command);
										openCommandsPopover = false;
									}}
								>
									<div class="flex items-center gap-2">
										<Icon
											icon={commandIcon ?? extensionIcon ?? undefined}
											class="mr-2 size-[18px]"
										/>
										<span>{command.title}</span>
									</div>
								</Command.Item>
							{/each}
						</Command.List>
					</Command.Root>
				</Popover.Content>
			</Popover.Root>
		{:else}
			<Button {...props} onclick={onInstall} disabled={isInstalling}>
				{isInstalling ? 'Installing...' : 'Install Extension'}
				<KeyboardShortcut shortcut={{ key: 'enter', modifiers: [] }} />
			</Button>
		{/if}
	{/snippet}
</ActionBar>
