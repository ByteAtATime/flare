import { invoke } from '@tauri-apps/api/core';

export type ScriptMode = 'fullOutput' | 'compact' | 'silent' | 'inline';

export type ScriptArgument = {
	name: string;
	placeholder?: string;
	optional: boolean;
	percentEncoded: boolean;
};

export type ScriptCommand = {
	path: string;
	filename: string;
	title: string;
	mode: ScriptMode;
	schemaVersion: number;
	packageName?: string;
	icon?: string;
	authors?: string;
	description?: string;
	arguments: ScriptArgument[];
	needsConfirmation: boolean;
};

class ScriptCommandsStore {
	commands = $state<ScriptCommand[]>([]);

	constructor() {
		this.loadCommands();
	}

	async loadCommands() {
		try {
			this.commands = await invoke('get_script_commands');
		} catch (error) {
			console.error('Failed to load script commands:', error);
		}
	}

	async runCommand(command: ScriptCommand, args: string[]) {
		try {
			const result = await invoke<string>('run_script_command', {
				commandPath: command.path,
				args
			});
			return result;
		} catch (error) {
			console.error('Failed to run script command:', error);
			throw error;
		}
	}

    async openScriptsFolder() {
        await invoke('open_scripts_folder');
    }
}

export const scriptCommandsStore = new ScriptCommandsStore();
