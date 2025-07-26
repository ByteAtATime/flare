import React from 'react';
import { updateContainer } from './reconciler';
import { writeLog, writeOutput } from './io';
import { getRaycastApi } from './api';
import { inspect } from 'util';
import * as fs from 'fs';
import * as path from 'path';
import type { PluginInfo, Preference } from '@flare/protocol';
import { environment } from './api/environment';
import { config } from './config';
import * as ReactJsxRuntime from 'react/jsx-runtime';
import { aiContext, setCurrentPlugin } from './state';

const createPluginRequire =
	() =>
	(moduleName: string): unknown => {
		if (moduleName === 'react') {
			return React;
		}

		if (moduleName.startsWith('@raycast/api')) {
			return getRaycastApi();
		}

		if (moduleName === 'react') {
			return React;
		}

		if (moduleName === 'react/jsx-runtime') {
			return ReactJsxRuntime;
		}

		return require(moduleName);
	};

export const loadPlugin = (pluginPath: string): string => {
	try {
		if (!fs.existsSync(pluginPath)) {
			throw new Error(`Plugin file not found: ${pluginPath}`);
		}
		return fs.readFileSync(pluginPath, 'utf-8');
	} catch (error) {
		writeLog(`Error loading plugin from ${pluginPath}: ${error}`);
		throw error;
	}
};

interface LaunchProps {
	arguments: Record<string, unknown>;
	launchType: typeof environment.launchType;
}

export const runPlugin = (
	pluginPath?: string,
	mode: 'view' | 'no-view' = 'view',
	aiAccessStatus: boolean
): void => {
	let pluginName = 'unknown';
	let preferences: Preference[] = [];

	if (!pluginPath) {
		throw new Error('No plugin specified.');
	}

	aiContext.hasAccess = aiAccessStatus;
	const scriptText = loadPlugin(pluginPath);

	const pluginDir = path.dirname(pluginPath);
	const packageJsonPath = path.join(pluginDir, 'package.json');

	if (fs.existsSync(packageJsonPath)) {
		try {
			const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
			pluginName = packageJson.name || path.basename(pluginDir);
			const pluginPreferences = packageJson.preferences || [];
			const allCommandPreferences = (packageJson.commands || []).flatMap(
				(cmd: { preferences?: Preference[] }) => cmd.preferences || []
			);
			preferences = [...pluginPreferences, ...allCommandPreferences];
		} catch (error) {
			writeLog(`Error reading plugin package.json: ${error}`);
		}
	}

	environment.assetsPath = path.join(config.pluginsDir, pluginName, 'assets');
	environment.extensionName = pluginName;

	setCurrentPlugin(pluginName, preferences);

	const pluginModule = {
		exports: {} as {
			default: React.ComponentType<LaunchProps> | ((props: LaunchProps) => Promise<void>) | null;
		}
	};

	const scriptFunction = new Function(
		'require',
		'module',
		'exports',
		'React',
		'console',
		scriptText
	);

	const mockConsole = {
		log: (...args: unknown[]) => {
			writeLog('[plugin] log: ' + args.map((arg) => inspect(arg, { depth: null })).join(' '));
		},
		warn: (...args: unknown[]) => {
			writeLog('[plugin] warn: ' + args.map((arg) => inspect(arg, { depth: null })).join(' '));
		},
		error: (...args: unknown[]) => {
			writeLog('[plugin] error: ' + args.map((arg) => inspect(arg, { depth: null })).join(' '));
		}
	};

	scriptFunction(createPluginRequire(), pluginModule, pluginModule.exports, React, mockConsole);

	const PluginRoot = pluginModule.exports.default;

	if (!PluginRoot) {
		throw new Error('Plugin did not export a default component.');
	}

	const launchProps: LaunchProps = {
		arguments: {},
		launchType: environment.launchType
	};

	if (mode === 'no-view') {
		if (typeof PluginRoot === 'function') {
			(PluginRoot as (props: LaunchProps) => Promise<void>)(launchProps)
				.then(() => {
					writeLog('No-view command finished.');
					writeOutput({ type: 'go-back-to-plugin-list', payload: {} });
				})
				.catch((e) => {
					writeLog(`No-view command failed: ${e}`);
					writeOutput({ type: 'go-back-to-plugin-list', payload: {} });
				});
		} else {
			throw new Error('No-view command did not export a default function.');
		}
	} else {
		writeLog('Plugin loaded. Initializing React render...');
		const AppElement = React.createElement(PluginRoot as React.ComponentType, launchProps);
		updateContainer(AppElement, () => {
			writeLog('Initial render complete');
		});
	}
};
