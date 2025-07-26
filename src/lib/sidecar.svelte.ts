import { Command, type Child, open as shellOpen } from '@tauri-apps/plugin-shell';
import { Unpackr } from 'msgpackr';
import { uiStore } from '$lib/ui.svelte';
import { CommandSchema, SidecarMessageWithPluginsSchema, type ImageLike } from '@flare/protocol';
import { invoke } from '@tauri-apps/api/core';
import { appCacheDir, appLocalDataDir } from '@tauri-apps/api/path';
import { listen } from '@tauri-apps/api/event';
import { imperativeBus } from './imperative.svelte';
import { inflate } from 'pako';

type OauthState = {
	url: string;
	providerName: string;
	providerIcon?: ImageLike;
	description?: string;
} | null;

class SidecarService {
	#sidecarChild: Child | null = $state(null);
	#receiveBuffer = Buffer.alloc(0);
	#unpackr = new Unpackr();
	#onGoBackToPluginList: (() => void) | null = null;
	#browserExtensionConnectionInterval: ReturnType<typeof setInterval> | null = null;
	#aiEventUnlisten: (() => void)[] = [];

	oauthState: OauthState = $state(null);
	logs: string[] = $state([]);

	constructor() {}

	get isRunning() {
		return this.#sidecarChild !== null;
	}

	setOnGoBackToPluginList(callback: () => void) {
		this.#onGoBackToPluginList = callback;
	}

	start = async () => {
		if (this.#sidecarChild) {
			this.#log('Sidecar service is already running.');
			return;
		}

		this.#log('Starting sidecar service...');
		try {
			const args: string[] = [];
			args.push(`--data-dir=${await appLocalDataDir()}`);
			args.push(`--cache-dir=${await appCacheDir()}`);
			console.log(args);
			const command = Command.sidecar('binaries/app', args.length > 0 ? args : undefined, {
				encoding: 'raw'
			});

			command.stdout.on('data', this.#handleStdout);
			command.stderr.on('data', (line) => this.#log(`STDERR: ${line}`));

			this.#sidecarChild = await command.spawn();
			this.#log(`Sidecar spawned with PID: ${this.#sidecarChild.pid}`);

			this.#setupAiEventListeners();

			this.#browserExtensionConnectionInterval = setInterval(async () => {
				try {
					const isConnected = await invoke<boolean>('browser_extension_check_connection');
					this.dispatchEvent('browser-extension-connection-status', { isConnected });
				} catch (e) {
					this.#log(`Error checking browser extension connection: ${e}`);
					this.dispatchEvent('browser-extension-connection-status', { isConnected: false });
				}
			}, 5000);
		} catch (e) {
			this.#log(`ERROR starting sidecar: ${e}`);
			console.error('Failed to start sidecar:', e);
		}
	};

	stop = () => {
		if (this.#sidecarChild) {
			this.#log('Stopping sidecar service...');
			this.#sidecarChild.kill();
			this.#sidecarChild = null;
		}
		if (this.#browserExtensionConnectionInterval) {
			clearInterval(this.#browserExtensionConnectionInterval);
			this.#browserExtensionConnectionInterval = null;
		}
		this.#aiEventUnlisten.forEach((unlisten) => unlisten());
		this.#aiEventUnlisten = [];
	};

	#setupAiEventListeners = async () => {
		try {
			const chunkUnlisten = await listen('ai-stream-chunk', (event) => {
				console.log(event.payload);
				this.dispatchEvent('ai-stream-chunk', event.payload as object);
			});

			const endUnlisten = await listen('ai-stream-end', (event) => {
				this.dispatchEvent('ai-stream-end', event.payload as object);
			});

			const errorUnlisten = await listen('ai-stream-error', (event) => {
				this.dispatchEvent('ai-stream-error', event.payload as object);
			});

			this.#aiEventUnlisten.push(chunkUnlisten, endUnlisten, errorUnlisten);
		} catch (error) {
			this.#log(`Error setting up AI event listeners: ${error}`);
		}
	};

	dispatchEvent = (action: string, payload?: object) => {
		if (!this.#sidecarChild) {
			this.#log('Cannot dispatch event, sidecar is not running.');
			return;
		}
		const message = JSON.stringify({ action, payload });
		this.#sidecarChild.write(message + '\n');
	};

	getPreferences = (pluginName: string) => {
		this.dispatchEvent('get-preferences', { pluginName });
	};

	setPreferences = (pluginName: string, values: Record<string, unknown>) => {
		this.dispatchEvent('set-preferences', { pluginName, values });
	};

	#handleStdout = (chunk: Uint8Array) => {
		try {
			this.#receiveBuffer = Buffer.concat([this.#receiveBuffer, Buffer.from(chunk)]);
			this.#processReceiveBuffer();
		} catch (e) {
			this.#log(`ERROR processing stdout: ${e}`);
			console.error('Failed to parse sidecar message:', chunk, e);
		}
	};

	#processReceiveBuffer = () => {
		while (this.#receiveBuffer.length >= 4) {
			const headerValue = this.#receiveBuffer.readUInt32BE(0);
			const isCompressed = (headerValue & 0x80000000) !== 0;
			const messageLength = headerValue & 0x7fffffff; // remove compression bit
			const totalLength = 4 + messageLength;

			if (this.#receiveBuffer.length >= totalLength) {
				let messagePayload: Uint8Array = this.#receiveBuffer.subarray(4, totalLength);
				this.#receiveBuffer = this.#receiveBuffer.subarray(totalLength);

				try {
					if (isCompressed) {
						messagePayload = inflate(messagePayload);
					}
					const message = this.#unpackr.unpack(messagePayload);
					this.#routeMessage(message);
				} catch (e) {
					console.error('Failed to unpack sidecar message:', e);
					this.#log(`ERROR unpacking message: ${e}`);
				}
			} else {
				break;
			}
		}
	};

	private messageParsingTimes: number[] = [];

	#routeMessage = async (message: unknown) => {
		const result = SidecarMessageWithPluginsSchema.safeParse(message);

		if (!result.success) {
			this.#log(`ERROR: Received invalid message from sidecar: ${result.error.message}`);
			console.error('Invalid sidecar message:', message);
			return;
		}

		const typedMessage = result.data;
		this.messageParsingTimes.push(Date.now() - typedMessage.timestamp);
		console.log(
			`Rolling average: ${this.messageParsingTimes.reduce((a, b) => a + b, 0) / this.messageParsingTimes.length}ms`
		);

		if (typedMessage.type === 'log') {
			this.#log(`SIDECAR: ${typedMessage.payload}`);
			return;
		}

		if (typedMessage.type === 'FOCUS_ELEMENT') {
			imperativeBus.dispatch(typedMessage.payload.elementId, 'focus');
			return;
		}

		if (typedMessage.type === 'RESET_ELEMENT') {
			imperativeBus.dispatch(typedMessage.payload.elementId, 'reset');
			return;
		}

		if (typedMessage.type === 'SHOW_HUD') {
			invoke('show_hud', { title: typedMessage.payload.title });
			return;
		}

		if (typedMessage.type === 'open') {
			const { target, application } = typedMessage.payload;
			shellOpen(target, application).catch((err) => {
				this.#log(`ERROR: Failed to open '${target}': ${err}`);
				console.error(`Failed to open '${target}':`, err);
			});
			return;
		}

		if (typedMessage.type === 'invoke_command') {
			const { requestId, command, params } = typedMessage.payload;
			const responseType = `invoke_command-response`;
			try {
				const result = await invoke(command, params);
				this.dispatchEvent(responseType, { requestId, result });
			} catch (error) {
				const errorMessage = error instanceof Error ? error.message : String(error);
				this.#log(`ERROR from ${command}: ${errorMessage}`);
				this.dispatchEvent(responseType, { requestId, error: errorMessage });
			}
			return;
		}

		if (typedMessage.type === 'ai-ask-stream') {
			const { requestId, prompt, options } = typedMessage.payload as {
				requestId: string;
				prompt: string;
				options: {
					model?: string;
					creativity?: string;
					modelMappings?: Record<string, string>;
				};
			};

			try {
				await invoke('ai_ask_stream', {
					requestId,
					prompt,
					options: {
						model: options.model,
						creativity: options.creativity,
						model_mappings: options.modelMappings || {}
					}
				});
			} catch (error) {
				const errorMessage = error instanceof Error ? error.message : String(error);
				this.#log(`ERROR from AI ask stream: ${errorMessage}`);
				this.dispatchEvent('ai-stream-error', { requestId, error: errorMessage });
			}
			return;
		}

		if (typedMessage.type === 'ai-can-access') {
			const { requestId } = typedMessage.payload as { requestId: string };
			const responseType = 'ai-can-access-response';
			try {
				const result = await invoke('ai_can_access');
				this.dispatchEvent(responseType, { requestId, result });
			} catch (error) {
				const errorMessage = error instanceof Error ? error.message : String(error);
				this.#log(`ERROR from ai_can_access: ${errorMessage}`);
				this.dispatchEvent(responseType, { requestId, error: errorMessage });
			}
			return;
		}

		if (typedMessage.type.startsWith('oauth-')) {
			if (typedMessage.type === 'oauth-authorize') {
				this.oauthState = typedMessage.payload;
				return;
			}

			const { requestId, ...params } = typedMessage.payload as {
				requestId: string;
				[key: string]: unknown;
			};

			const commandMap: Record<string, string> = {
				'oauth-get-tokens': 'oauth_get_tokens',
				'oauth-set-tokens': 'oauth_set_tokens',
				'oauth-remove-tokens': 'oauth_remove_tokens'
			};
			const command = commandMap[typedMessage.type];

			if (command) {
				const responseType = `${typedMessage.type}-response`;
				try {
					const result = await invoke(command, params);
					this.dispatchEvent(responseType, { requestId, result });
				} catch (error) {
					const errorMessage = error instanceof Error ? error.message : String(error);
					this.#log(`ERROR from ${command}: ${errorMessage}`);
					this.dispatchEvent(responseType, { requestId, error: errorMessage });
				}
				return;
			}
		}

		if (typedMessage.type === 'preference-values') {
			uiStore.setCurrentPreferences(typedMessage.payload.values);
			return;
		}

		if (typedMessage.type === 'go-back-to-plugin-list') {
			if (this.#onGoBackToPluginList) {
				this.#onGoBackToPluginList();
			}
			return;
		}

		if (typedMessage.type === 'browser-extension-request') {
			const { requestId, method, params } = typedMessage.payload;
			invoke('browser_extension_request', { method, params })
				.then((result) => {
					this.dispatchEvent('browser-extension-response', { requestId, result });
				})
				.catch((error) => {
					this.#log(`ERROR from browser extension request: ${error}`);
					this.dispatchEvent('browser-extension-response', { requestId, error: String(error) });
				});
			return;
		}

		if (typedMessage.type === 'BATCH_UPDATE') {
			uiStore.applyCommands(typedMessage.payload);
		} else {
			const parseResult = CommandSchema.safeParse(typedMessage);
			if (parseResult.success) {
				uiStore.applyCommands([parseResult.data]);
			}
		}
	};

	#log = (message: string) => {
		console.log(`[SidecarService] ${message}`);
		this.logs.push(message);
	};
}

export const sidecarService = new SidecarService();
