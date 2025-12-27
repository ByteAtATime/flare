import { invoke } from '@tauri-apps/api/core';

export type AiPreset = {
	id: string;
	name: string;
	template: string;
	icon: string | null;
	createdAt: number;
};

class AiStore {
	presets = $state<AiPreset[]>([]);

	constructor() {
		this.loadPresets();
	}

	async loadPresets() {
		try {
			this.presets = await invoke('get_ai_presets');
		} catch (error) {
			console.error('Failed to load AI presets:', error);
		}
	}

	async createPreset(name: string, template: string, icon: string | null) {
		try {
			const preset = await invoke<AiPreset>('create_ai_preset', { name, template, icon });
			this.presets.push(preset);
			// Re-sort locally or reload
			this.presets.sort((a, b) => a.name.localeCompare(b.name));
			return preset;
		} catch (error) {
			console.error('Failed to create AI preset:', error);
			throw error;
		}
	}

	async updatePreset(id: string, name: string, template: string, icon: string | null) {
		try {
			await invoke('update_ai_preset', { id, name, template, icon });
			const index = this.presets.findIndex((p) => p.id === id);
			if (index !== -1) {
				this.presets[index] = { ...this.presets[index], name, template, icon };
				this.presets.sort((a, b) => a.name.localeCompare(b.name));
			}
		} catch (error) {
			console.error('Failed to update AI preset:', error);
			throw error;
		}
	}

	async deletePreset(id: string) {
		try {
			await invoke('delete_ai_preset', { id });
			this.presets = this.presets.filter((p) => p.id !== id);
		} catch (error) {
			console.error('Failed to delete AI preset:', error);
			throw error;
		}
	}
}

export const aiStore = new AiStore();
