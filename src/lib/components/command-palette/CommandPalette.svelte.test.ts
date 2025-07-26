import { mockedClipboard, mockedCore } from '$lib/__mocks__/tauri.mock';
import { render, screen, cleanup, fireEvent, waitFor, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import CommandPalette from './CommandPalette.svelte';
import { type PluginInfo } from '@flare/protocol';
import type { App } from '$lib/apps.svelte';
import type { Quicklink } from '$lib/quicklinks.svelte';
import { focusManager } from '$lib/focus.svelte';

const appsStore = vi.hoisted(() => ({
	apps: [] as App[],
	isLoading: false
}));
vi.mock('$lib/apps.svelte', () => ({
	appsStore
}));

const quicklinksStore = vi.hoisted(() => ({
	quicklinks: [] as Quicklink[],
	isLoading: false,
	error: null
}));
vi.mock('$lib/quicklinks.svelte', () => ({
	quicklinksStore
}));

const frecencyStore = vi.hoisted(() => ({
	data: [],
	isLoading: false,
	hiddenItemIds: [],
	recordUsage: vi.fn().mockResolvedValue(undefined),
	hideItem: vi.fn().mockResolvedValue(undefined)
}));
vi.mock('$lib/frecency.svelte', () => ({
	frecencyStore
}));

const viewManager = vi.hoisted(() => ({
	showSettings: vi.fn()
}));
vi.mock('$lib/viewManager.svelte', () => ({
	viewManager
}));

describe('CommandPalette.svelte', () => {
	const onRunPlugin = vi.fn();
	const user = userEvent.setup();

	const mockPlugins: PluginInfo[] = [
		{
			pluginPath: '/path/to/plugin1',
			title: 'Mock Plugin 1',
			pluginName: 'mock-plugin-1',
			pluginTitle: 'Mock Extension',
			commandName: 'mock-command-1',
			icon: 'mock-icon-16',
			mode: 'view',
			owner: 'test',
			author: 'test',
			preferences: [],
			description: 'A mock plugin'
		},
		{
			pluginPath: '/path/to/plugin2',
			title: 'Mock Plugin 2',
			pluginName: 'mock-plugin-2',
			pluginTitle: 'Another Extension',
			commandName: 'mock-command-2',
			icon: 'mock-icon-16',
			mode: 'view',
			owner: 'test',
			preferences: [],
			description: 'Another mock plugin'
		}
	];

	const mockApps: App[] = [
		{
			name: 'Test App 1',
			comment: 'A great testing application',
			exec: '/usr/bin/test-app-1'
		}
	];

	const mockQuicklinks: Quicklink[] = [
		{
			id: 1,
			name: 'Google Search',
			link: 'https://google.com/search?q={argument}',
			application: null,
			icon: 'link-16',
			createdAt: new Date().toISOString(),
			updatedAt: new Date().toISOString()
		},
		{
			id: 2,
			name: 'Simple Link',
			link: 'https://example.com',
			application: null,
			icon: 'link-16',
			createdAt: new Date().toISOString(),
			updatedAt: new Date().toISOString()
		}
	];

	beforeEach(() => {
		cleanup();
		vi.clearAllMocks();
		appsStore.apps = [];
		quicklinksStore.quicklinks = [];
		frecencyStore.data = [];
		mockedCore.invoke.mockResolvedValue(undefined);
		focusManager.reset();
	});

	describe('1. Initial Rendering and Display', () => {
		it('should render the search input and default placeholder', () => {
			render(CommandPalette, {
				plugins: [],
				onRunPlugin
			});

			const input = screen.getByPlaceholderText('Search for apps and commands...');
			expect(input).toBeInTheDocument();

			expect(input).toHaveFocus();
		});

		it('should display a list of plugins', async () => {
			render(CommandPalette, {
				plugins: mockPlugins,
				onRunPlugin
			});

			expect(await screen.findByText('Mock Plugin 1')).toBeInTheDocument();
			expect(await screen.findByText('Mock Extension')).toBeInTheDocument();
			expect(await screen.findByText('Mock Plugin 2')).toBeInTheDocument();
			expect(await screen.findByText('Another Extension')).toBeInTheDocument();

			const commandAccessories = await screen.findAllByText('Command');
			expect(commandAccessories.length).toBe(2);
		});

		it('should display a list of installed applications', async () => {
			appsStore.apps = mockApps;

			render(CommandPalette, { plugins: [], onRunPlugin });

			expect(await screen.findByText('Test App 1')).toBeInTheDocument();
			expect(await screen.findByText('A great testing application')).toBeInTheDocument();

			const appAccessories = await screen.findAllByText('Application');
			expect(appAccessories.length).toBe(1);
		});

		it('should display a list of quicklinks', async () => {
			quicklinksStore.quicklinks = mockQuicklinks;

			render(CommandPalette, { plugins: [], onRunPlugin });

			expect(await screen.findByText('Google Search', { selector: 'p' })).toBeInTheDocument();
			expect(await screen.findByText('https://google.com/search?q=...')).toBeInTheDocument();

			const quicklinkAccessories = await screen.findAllByText('Quicklink');
			expect(quicklinkAccessories.length).toBe(2);
		});
	});

	describe('2. Search and Filtering', () => {
		it('should filter the list based on search text', async () => {
			appsStore.apps = mockApps;
			quicklinksStore.quicklinks = mockQuicklinks;

			const { container } = render(CommandPalette, {
				plugins: mockPlugins,
				onRunPlugin
			});

			const listContainer = within(container).getByTestId('command-palette-content');

			const initialListCount = within(listContainer).getAllByTestId('list-item').length;

			const searchInput = screen.getByPlaceholderText('Search for apps and commands...');
			await user.type(searchInput, 'Mock Plugin 1');

			expect(listContainer).toBeInTheDocument();

			const listItems = within(listContainer).getAllByTestId('list-item');
			expect(listItems.length).not.toEqual(initialListCount);
			expect(listItems[0]).toHaveClass('!bg-accent');
		});

		it('should display the calculator result when a mathematical expression is typed', async () => {
			mockedCore.invoke.mockImplementation(async (command) => {
				if (command === 'calculate_soulver') {
					return JSON.stringify({ value: '4', type: 'number' });
				}
			});

			render(CommandPalette, { plugins: [], onRunPlugin });

			const searchInput = screen.getByPlaceholderText('Search for apps and commands...');
			await user.type(searchInput, '2+2');

			expect(await screen.findByText('Calculator')).toBeInTheDocument();
			expect(screen.getByText('2+2')).toBeInTheDocument();
			expect(screen.getByText('4')).toBeInTheDocument();
		});

		it('should not display the calculator for non-mathematical text', async () => {
			mockedCore.invoke.mockImplementation(async (command) => {
				if (command === 'calculate_soulver') {
					return JSON.stringify({ type: 'none' });
				}
			});

			render(CommandPalette, { plugins: [], onRunPlugin });

			const searchInput = screen.getByPlaceholderText('Search for apps and commands...');
			await user.type(searchInput, 'hello world');

			expect(screen.queryByText('Calculator')).not.toBeInTheDocument();
		});

		it('should clear the search text when the Escape key is pressed in the input', async () => {
			render(CommandPalette, { plugins: [], onRunPlugin });
			const searchInput = screen.getByPlaceholderText<HTMLInputElement>(
				'Search for apps and commands...'
			);

			await user.type(searchInput, 'some text');
			expect(searchInput.value).toBe('some text');

			await fireEvent.keyDown(searchInput, { key: 'Escape' });

			expect(searchInput.value).toBe('');
		});
	});

	describe('3. Item Selection and Execution (`Enter` key)', () => {
		it('should execute a selected plugin on Enter', async () => {
			render(CommandPalette, { plugins: [mockPlugins[0]], onRunPlugin });

			await user.keyboard('{Enter}');

			expect(onRunPlugin).toHaveBeenCalledTimes(1);
			expect(onRunPlugin).toHaveBeenCalledWith(mockPlugins[0]);
			expect(frecencyStore.recordUsage).toHaveBeenCalledWith(mockPlugins[0].pluginPath);
		});

		it('should launch a selected application on Enter', async () => {
			appsStore.apps = mockApps;
			render(CommandPalette, { plugins: [], onRunPlugin });

			await user.keyboard('{Enter}');

			expect(mockedCore.invoke).toHaveBeenCalledWith('launch_app', { exec: mockApps[0].exec });
			expect(frecencyStore.recordUsage).toHaveBeenCalledWith(mockApps[0].exec);
		});

		it('should copy the calculator answer on Enter', async () => {
			mockedCore.invoke.mockImplementation(async (command) => {
				if (command === 'calculate_soulver') {
					return JSON.stringify({ value: '4', type: 'number' });
				}
			});

			render(CommandPalette, { plugins: [], onRunPlugin });
			const searchInput = screen.getByPlaceholderText('Search for apps and commands...');
			await user.type(searchInput, '2+2');

			await waitFor(() => expect(screen.getByText('Calculator')).toBeInTheDocument());

			await user.keyboard('{Enter}');

			expect(mockedClipboard.writeText).toHaveBeenCalledWith('4');
		});

		it('should execute a simple quicklink (without argument) on Enter', async () => {
			const simpleQuicklink = mockQuicklinks.find((q) => !q.link.includes('{argument}'))!;
			quicklinksStore.quicklinks = [simpleQuicklink];

			render(CommandPalette, { plugins: [], onRunPlugin });

			await waitFor(() => expect(screen.getByText(simpleQuicklink.name)).toBeInTheDocument());

			await user.keyboard('{Enter}');

			expect(mockedCore.invoke).toHaveBeenCalledWith('execute_quicklink', {
				link: simpleQuicklink.link,
				application: simpleQuicklink.application
			});
			expect(frecencyStore.recordUsage).toHaveBeenCalledWith(`quicklink-${simpleQuicklink.id}`);
		});
	});

	describe('4. Quicklinks with Arguments', () => {
		it('should show the argument input when a complex quicklink is selected', async () => {
			const complexQuicklink = mockQuicklinks.find((q) => q.link.includes('{argument}'))!;
			quicklinksStore.quicklinks = [complexQuicklink];

			render(CommandPalette, { plugins: [], onRunPlugin });

			const argumentInput = await screen.findByPlaceholderText('Query');
			expect(argumentInput).toBeInTheDocument();

			await user.keyboard('{Enter}');
			await waitFor(() => expect(argumentInput).toHaveFocus());

			const mainInput = screen.getByPlaceholderText(complexQuicklink.name);
			expect(mainInput).toBeInTheDocument();
		});

		it('should execute a complex quicklink with the provided argument', async () => {
			const complexQuicklink = mockQuicklinks.find((q) => q.link.includes('{argument}'))!;
			quicklinksStore.quicklinks = [complexQuicklink];

			render(CommandPalette, { plugins: [], onRunPlugin });

			const argumentInput = await screen.findByPlaceholderText('Query');
			expect(argumentInput).toBeInTheDocument();

			await user.keyboard('{Enter}');
			expect(argumentInput).toHaveFocus();

			await user.type(argumentInput, 'Svelte');
			await user.keyboard('{Enter}');

			expect(mockedCore.invoke).toHaveBeenCalledWith('execute_quicklink', {
				link: 'https://google.com/search?q=Svelte',
				application: complexQuicklink.application
			});

			const mainInput = screen.getByPlaceholderText('Search for apps and commands...');
			expect(mainInput).toHaveValue('');
		});

		it('should hide the argument input on Escape', async () => {
			const complexQuicklink = mockQuicklinks.find((q) => q.link.includes('{argument}'))!;
			quicklinksStore.quicklinks = [complexQuicklink];

			render(CommandPalette, { plugins: [], onRunPlugin });

			const argumentInput = await screen.findByPlaceholderText('Query');
			await user.keyboard('{Enter}');
			await waitFor(() => expect(argumentInput).toHaveFocus());

			await user.keyboard('{Escape}');

			const mainInput = screen.getByPlaceholderText(complexQuicklink.name);
			expect(mainInput).toHaveFocus();
		});

		it('should hide the argument input on Backspace in an empty argument input', async () => {
			const complexQuicklink = mockQuicklinks.find((q) => q.link.includes('{argument}'))!;
			quicklinksStore.quicklinks = [complexQuicklink];

			render(CommandPalette, { plugins: [], onRunPlugin });

			const argumentInput = await screen.findByPlaceholderText('Query');
			await user.keyboard('{Enter}');
			await waitFor(() => expect(argumentInput).toHaveFocus());

			expect(argumentInput).toHaveValue('');
			await user.keyboard('{Backspace}');

			const mainInput = screen.getByPlaceholderText(complexQuicklink.name);
			expect(mainInput).toHaveFocus();
		});
	});

	describe('5. Keyboard Shortcuts and Actions', () => {
		it('should trigger "Copy Deeplink" with Ctrl+Shift+C for a plugin', async () => {
			render(CommandPalette, { plugins: [mockPlugins[0]], onRunPlugin });
			const searchInput = await screen.findByPlaceholderText('Search for apps and commands...');
			await waitFor(() => expect(screen.getByText('Mock Plugin 1')).toBeInTheDocument());

			await fireEvent.keyDown(searchInput, { key: 'c', ctrlKey: true, shiftKey: true });

			expect(mockedClipboard.writeText).toHaveBeenCalledWith(
				'raycast://extensions/test/mock-plugin-1/mock-command-1'
			);
		});

		it('should trigger "Copy App Path" with Ctrl+Shift+. for an app', async () => {
			appsStore.apps = mockApps;
			render(CommandPalette, { plugins: [], onRunPlugin });
			const searchInput = await screen.findByPlaceholderText('Search for apps and commands...');
			await waitFor(() => expect(screen.getByText('Test App 1')).toBeInTheDocument());

			await fireEvent.keyDown(searchInput, { key: '.', ctrlKey: true, shiftKey: true });

			expect(mockedClipboard.writeText).toHaveBeenCalledWith('/usr/bin/test-app-1');
		});

		it('should trigger "Hide Application" with Ctrl+h for an app', async () => {
			appsStore.apps = mockApps;
			render(CommandPalette, { plugins: [], onRunPlugin });
			const searchInput = await screen.findByPlaceholderText('Search for apps and commands...');
			await waitFor(() => expect(screen.getByText('Test App 1')).toBeInTheDocument());

			await fireEvent.keyDown(searchInput, { key: 'h', ctrlKey: true });

			expect(frecencyStore.hideItem).toHaveBeenCalledWith('/usr/bin/test-app-1');
		});

		it('should trigger "Configure Command" with Ctrl+Shift+, for a plugin', async () => {
			render(CommandPalette, { plugins: [mockPlugins[0]], onRunPlugin });
			const searchInput = await screen.findByPlaceholderText('Search for apps and commands...');
			await waitFor(() => expect(screen.getByText('Mock Plugin 1')).toBeInTheDocument());

			await fireEvent.keyDown(searchInput, { key: ',', ctrlKey: true, shiftKey: true });

			expect(viewManager.showSettings).toHaveBeenCalledWith('mock-plugin-1');
		});
	});

	describe('6. Keyboard Navigation', () => {
		it('should move selection down with ArrowDown key', async () => {
			render(CommandPalette, { plugins: mockPlugins, onRunPlugin });
			const listItems = await screen.findAllByTestId('list-item');
			expect(listItems[0]).toHaveClass('!bg-accent');

			await user.keyboard('{ArrowDown}');

			expect(listItems[0]).not.toHaveClass('!bg-accent');
			expect(listItems[1]).toHaveClass('!bg-accent');
		});

		it('should move selection up with ArrowUp key', async () => {
			render(CommandPalette, { plugins: mockPlugins, onRunPlugin });
			const listItems = await screen.findAllByTestId('list-item');
			await user.keyboard('{ArrowDown}');

			expect(listItems[1]).toHaveClass('!bg-accent');

			await user.keyboard('{ArrowUp}');

			expect(listItems[1]).not.toHaveClass('!bg-accent');
			expect(listItems[0]).toHaveClass('!bg-accent');
		});

		it('should wrap selection from top to bottom on ArrowUp', async () => {
			render(CommandPalette, { plugins: mockPlugins, onRunPlugin });
			const listItems = await screen.findAllByTestId('list-item');
			expect(listItems[0]).toHaveClass('!bg-accent');

			await user.keyboard('{ArrowUp}');

			expect(listItems[0]).not.toHaveClass('!bg-accent');
			expect(listItems[listItems.length - 1]).toHaveClass('!bg-accent');
		});

		it('should wrap selection from bottom to top on ArrowDown', async () => {
			render(CommandPalette, { plugins: mockPlugins, onRunPlugin });
			const listItems = await screen.findAllByTestId('list-item');

			for (let i = 0; i < listItems.length - 1; i++) {
				await user.keyboard('{ArrowDown}');
			}

			expect(listItems[listItems.length - 1]).toHaveClass('!bg-accent');

			await user.keyboard('{ArrowDown}');

			expect(listItems[listItems.length - 1]).not.toHaveClass('!bg-accent');
			expect(listItems[0]).toHaveClass('!bg-accent');
		});
	});
});
