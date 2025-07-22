import { LaunchType } from './types';
import * as fs from 'fs';
import { writeOutput } from '../io';
import type { Application } from './types';
import { config } from '../config';
import { browserExtensionState, aiContext } from '../state';
import { invokeCommand } from './rpc';

const supportPath = config.supportDir;
try {
	if (!fs.existsSync(supportPath)) {
		fs.mkdirSync(supportPath, { recursive: true });
	}
} catch (e) {
	console.error('Could not create support path', e);
}

export interface FileSystemItem {
	path: string;
}

export const BrowserExtension = { name: 'BrowserExtension' };
export const AI = { name: 'AI' };

export const environment = {
	appearance: 'dark' as const,
	assetsPath: config.assetsDir,
	commandMode: 'view' as const,
	commandName: 'index',
	extensionName: 'my-extension',
	isDevelopment: true,
	launchType: LaunchType.UserInitiated,
	ownerOrAuthorName: 'Flare',
	raycastVersion: '1.0.0',
	supportPath: supportPath,
	textSize: 'medium' as const,
	canAccess: (feature: { name: string }): boolean => {
		if (feature && feature.name === 'BrowserExtension') {
			return browserExtensionState.isConnected;
		}
		if (feature && feature.name === 'AI') {
			return aiContext.hasAccess;
		}
		return true;
	}
};

export async function getSelectedFinderItems(): Promise<FileSystemItem[]> {
	return invokeCommand<FileSystemItem[]>('get_selected_finder_items');
}

export async function getSelectedText(): Promise<string> {
	return invokeCommand<string>('get_selected_text');
}

export async function open(target: string, application?: Application | string): Promise<void> {
	let openWith: string | undefined;

	if (typeof application === 'string') {
		openWith = application;
	} else if (application) {
		openWith = application.path;
	}

	writeOutput({
		type: 'open',
		payload: {
			target,
			application: openWith
		}
	});
}

export async function getApplications(path?: fs.PathLike): Promise<Application[]> {
	const pathString = path ? path.toString() : undefined;
	return invokeCommand<Application[]>('get_applications', { path: pathString });
}

export async function getDefaultApplication(path: fs.PathLike): Promise<Application> {
	return invokeCommand<Application>('get_default_application', { path: path.toString() });
}

export async function getFrontmostApplication(): Promise<Application> {
	return invokeCommand<Application>('get_frontmost_application');
}

export async function showInFinder(path: fs.PathLike): Promise<void> {
	return invokeCommand<void>('show_in_finder', { path: path.toString() });
}

export async function trash(path: fs.PathLike | fs.PathLike[]): Promise<void> {
	const paths = (Array.isArray(path) ? path : [path]).map((p) => p.toString());
	return invokeCommand<void>('trash', { paths });
}
