import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
    // Appearance
    theme: 'light' | 'dark' | 'system';
    windowOpacity: number;
    fontSize: 'small' | 'medium' | 'large';

    // Search Settings
    enableSearchHistory: boolean;
    searchResultsLimit: number;
    fuzzySearchSensitivity: 'low' | 'medium' | 'high';

    // Window Behavior
    closeOnBlur: boolean;
    rememberWindowPosition: boolean;
    defaultWindowWidth: number;
    defaultWindowHeight: number;

    // Developer Options
    developerMode: boolean;
    showExtensionConsole: boolean;
    debugLogLevel: 'error' | 'warn' | 'info' | 'debug' | 'trace';

    // Performance
    maxConcurrentExtensions: number;
    cacheSizeMb: number;
    indexingThrottleMs: number;

    // System Integration
    autoStartOnLogin: boolean;
    clipboardHistoryRetentionDays: number;
}

class SettingsStore {
    settings = $state<AppSettings>({
        // Appearance
        theme: 'system',
        windowOpacity: 1.0,
        fontSize: 'medium',

        // Search Settings
        enableSearchHistory: true,
        searchResultsLimit: 50,
        fuzzySearchSensitivity: 'medium',

        // Window Behavior
        closeOnBlur: false,
        rememberWindowPosition: true,
        defaultWindowWidth: 800,
        defaultWindowHeight: 600,

        // Developer Options
        developerMode: false,
        showExtensionConsole: false,
        debugLogLevel: 'info',

        // Performance
        maxConcurrentExtensions: 5,
        cacheSizeMb: 100,
        indexingThrottleMs: 500,

        // System Integration
        autoStartOnLogin: false,
        clipboardHistoryRetentionDays: 30
    });

    private loaded = $state(false);

    async loadSettings() {
        try {
            const settings = await invoke<AppSettings>('get_app_settings');
            this.settings = settings;
            this.loaded = true;
        } catch (error) {
            console.error('Failed to load app settings:', error);
            // Keep default values
            this.loaded = true;
        }
    }

    async saveSettings() {
        try {
            await invoke('save_app_settings', { settings: this.settings });
        } catch (error) {
            console.error('Failed to save app settings:', error);
            throw error;
        }
    }

    async updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
        this.settings[key] = value;
        await this.saveSettings();
    }

    async resetToDefaults() {
        try {
            const defaults = await invoke<AppSettings>('reset_app_settings');
            this.settings = defaults;
        } catch (error) {
            console.error('Failed to reset app settings:', error);
            throw error;
        }
    }

    isLoaded() {
        return this.loaded;
    }
}

export const settingsStore = new SettingsStore();
