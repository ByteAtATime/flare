import { invoke } from '@tauri-apps/api/core';

class AliasesStore {
	aliases = $state<Record<string, string>>({});

	constructor() {
		this.loadAliases();
	}

	async loadAliases() {
		try {
			this.aliases = await invoke('get_aliases');
		} catch (error) {
			console.error('Failed to load aliases:', error);
		}
	}

	async setAlias(alias: string, commandId: string) {
		try {
			await invoke('set_alias', { alias, commandId });
			this.aliases = { ...this.aliases, [alias]: commandId };
		} catch (error) {
			console.error('Failed to set alias:', error);
			throw error;
		}
	}

	async removeAlias(alias: string) {
		try {
			await invoke('remove_alias', { alias });
			const newAliases = { ...this.aliases };
			delete newAliases[alias];
			this.aliases = newAliases;
		} catch (error) {
			console.error('Failed to remove alias:', error);
			throw error;
		}
	}

    getCommandId(alias: string): string | undefined {
        return this.aliases?.[alias];
    }
}

export const aliasesStore = new AliasesStore();
