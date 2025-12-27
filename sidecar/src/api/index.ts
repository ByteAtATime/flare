import React from 'react';
import { Color } from './colors';
import { Cache } from './cache';
import { Icon } from './icon';
import { LaunchType, Toast } from './types';
import { createLocalStorage } from './utils';
import { useNavigation } from './navigation';
import { List } from './components/list';
import { Grid } from './components/grid';
import { Form } from './components/form';
import { Action, ActionPanel } from './components/actions';
import { Detail } from './components/detail';
import {
	environment,
	getSelectedFinderItems,
	getSelectedText,
	open,
	getApplications,
	getDefaultApplication,
	getFrontmostApplication,
	showInFinder,
	trash,
	runAppleScript,
	AI as AIConstant
} from './environment';
import { preferencesStore } from '../preferences';
import { showToast } from './toast';
import { showHUD } from './hud';
import { BrowserExtensionAPI } from './browserExtension';
import { Clipboard } from './clipboard';
import * as OAuth from './oauth';
import { AI } from './ai';
import { Keyboard } from './keyboard';
import { currentPluginName, currentPluginPreferences } from '../state';
import { writeOutput } from '../io';

const Image = {
	Mask: {
		Circle: 'circle',
		RoundedRectangle: 'roundedRectangle'
	}
};

export const getRaycastApi = () => {
	const LocalStorage = createLocalStorage();

	return {
		LocalStorage,
		Color,
		Cache,
		Icon,
		Image,
		LaunchType,
		Toast,
		OAuth,
		AI: {
			...AI,
			...AIConstant
		},
		Action,
		ActionPanel,
		Detail,
		Form,
		Grid,
		List,
		Clipboard,
		environment,
		getApplications,
		getDefaultApplication,
		getFrontmostApplication,
		getPreferenceValues: () => {
			if (currentPluginName) {
				return preferencesStore.getPreferenceValues(currentPluginName, currentPluginPreferences);
			}
			return {};
		},
		getSelectedFinderItems,
		getSelectedText,
		open,
		showInFinder,
		showToast,
		showHUD,
		trash,
		runAppleScript,
		closeMainWindow: async () => {
			// Send message to frontend to hide the main window
			writeOutput({
				type: 'close-main-window',
				payload: {}
			});
		},
		popToRoot: async () => {
			// Navigate back to plugin list - extensions handle this themselves
			// by completing execution which triggers go-back-to-plugin-list
		},
		useNavigation,
		usePersistentState: <T>(
			key: string,
			initialValue: T
		): [T, React.Dispatch<React.SetStateAction<T>>, boolean] => {
			const [state, setState] = React.useState<T>(initialValue);
			const [isLoading, setIsLoading] = React.useState(true);

			// Load persisted value on mount
			React.useEffect(() => {
				LocalStorage.getItem(key)
					.then((stored) => {
						if (stored !== undefined) {
							try {
								setState(JSON.parse(stored));
							} catch (e) {
								console.error(`Failed to parse persisted state for key "${key}":`, e);
							}
						}
					})
					.catch((e) => {
						console.error(`Failed to load persisted state for key "${key}":`, e);
					})
					.finally(() => {
						setIsLoading(false);
					});
			}, [key]);

			// Wrapper that persists to LocalStorage on every state change
			const setPersistentState = React.useCallback(
				(value: React.SetStateAction<T>) => {
					setState((prev) => {
						const nextValue = typeof value === 'function' ? (value as (prev: T) => T)(prev) : value;
						LocalStorage.setItem(key, JSON.stringify(nextValue)).catch((e) => {
							console.error(`Failed to persist state for key "${key}":`, e);
						});
						return nextValue;
					});
				},
				[key]
			);

			return [state, setPersistentState, isLoading];
		},
		BrowserExtension: BrowserExtensionAPI,
		Keyboard
	};
};
