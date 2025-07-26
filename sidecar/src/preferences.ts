import * as fs from 'fs';
import * as path from 'path';
import { writeLog } from './io';
import type { Preference } from '@flare/protocol';
import { config } from './config';

export class PreferencesStore {
	private preferencesPath: string;
	private preferences: Record<string, Record<string, unknown>> = {};

	constructor() {
		this.preferencesPath = config.preferencesFile;
		this.loadPreferences();
	}

	private loadPreferences(): void {
		try {
			if (fs.existsSync(this.preferencesPath)) {
				const data = fs.readFileSync(this.preferencesPath, 'utf-8');
				this.preferences = JSON.parse(data);
			}
		} catch (error) {
			writeLog(`Error loading preferences: ${error}`);
			this.preferences = {};
		}
	}

	private savePreferences(): void {
		try {
			const dir = path.dirname(this.preferencesPath);
			fs.mkdirSync(dir, { recursive: true });
			fs.writeFileSync(this.preferencesPath, JSON.stringify(this.preferences, null, 2));
		} catch (error) {
			writeLog(`Error saving preferences: ${error}`);
		}
	}

	public getPreferenceValues(
		pluginName: string,
		preferenceDefinitions?: Preference[]
	): Record<string, unknown> {
		const pluginPrefs = this.preferences[pluginName] || {};
		const result: Record<string, unknown> = {};

		if (preferenceDefinitions) {
			for (const pref of preferenceDefinitions) {
				if (pluginPrefs[pref.name] !== undefined) {
					result[pref.name] = pluginPrefs[pref.name];
				} else if (pref.default !== undefined) {
					result[pref.name] = pref.default;
				}
			}
		}

		return result;
	}

	public setPreferenceValues(pluginName: string, values: Record<string, unknown>): void {
		if (!this.preferences[pluginName]) {
			this.preferences[pluginName] = {};
		}

		Object.assign(this.preferences[pluginName], values);
		this.savePreferences();
	}

	public getAllPreferences(): Record<string, Record<string, unknown>> {
		return { ...this.preferences };
	}
}

export const preferencesStore = new PreferencesStore();
