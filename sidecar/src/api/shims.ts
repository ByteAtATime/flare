import { invokeCommand } from './rpc';

/**
 * Extension shims for macOS API compatibility on Linux
 * This module provides Linux equivalents for macOS-specific APIs
 */

export interface ShimResult {
    success: boolean;
    output?: string;
    error?: string;
}

/**
 * Translates macOS paths to Linux equivalents
 * @param path - macOS path to translate
 * @returns Translated Linux path
 */
export async function translatePath(path: string): Promise<string> {
    return invokeCommand<string>('shim_translate_path', { path });
}

/**
 * Attempts to execute AppleScript by translating to Linux equivalents
 * @param script - AppleScript code to execute
 * @returns Result of the shim execution
 */
export async function runAppleScript(script: string): Promise<ShimResult> {
    return invokeCommand<ShimResult>('shim_run_applescript', { script });
}

/**
 * Gets system information in a cross-platform way
 * @returns System information map
 */
export async function getSystemInfo(): Promise<Record<string, string>> {
    return invokeCommand<Record<string, string>>('shim_get_system_info', {});
}

/**
 * Normalizes file paths in extension code
 * Replaces macOS-specific paths with Linux equivalents
 * @param code - Extension code to normalize
 * @returns Normalized code
 */
export function normalizePathsInCode(code: string): string {
    let normalized = code;

    // Replace common macOS path patterns
    const pathReplacements: Array<[RegExp, string]> = [
        [/\/Applications\//g, '/usr/share/applications/'],
        [/\/Library\//g, '/usr/lib/'],
        [/\/Users\//g, '/home/'],
        [/~\/Library\/Application Support\//g, '~/.local/share/'],
        [/~\/Library\/Preferences\//g, '~/.config/'],
        [/~\/Library\//g, '~/.local/lib/']
    ];

    for (const [pattern, replacement] of pathReplacements) {
        normalized = normalized.replace(pattern, replacement);
    }

    return normalized;
}

/**
 * Shim for Raycast's runAppleScript API
 * This function should be injected into the extension environment
 */
export async function runAppleScriptShim(script: string): Promise<string> {
    const result = await runAppleScript(script);

    if (!result.success) {
        throw new Error(result.error || 'AppleScript execution failed');
    }

    return result.output || '';
}
