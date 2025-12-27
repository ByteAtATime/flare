import { invoke } from '@tauri-apps/api/core';

export type PowerCommand = 'shutdown' | 'restart' | 'sleep' | 'lock';

export interface VolumeLevel {
    percentage: number;
    isMuted: boolean;
}

/**
 * Execute a power management command
 */
export async function executePowerCommand(command: PowerCommand): Promise<void> {
    const normalizedCommand = command.charAt(0).toUpperCase() + command.slice(1);
    await invoke('execute_power_command', { command: normalizedCommand });
}

/**
 * Shut down the system
 */
export async function shutdown(): Promise<void> {
    await executePowerCommand('shutdown');
}

/**
 * Restart the system
 */
export async function restart(): Promise<void> {
    await executePowerCommand('restart');
}

/**
 * Put the system to sleep
 */
export async function sleep(): Promise<void> {
    await executePowerCommand('sleep');
}

/**
 * Lock the screen
 */
export async function lockScreen(): Promise<void> {
    await executePowerCommand('lock');
}

/**
 * Set system volume (0-100%)
 */
export async function setVolume(level: number): Promise<void> {
    const clampedLevel = Math.max(0, Math.min(100, level));
    await invoke('set_volume', { level: clampedLevel });
}

/**
 * Increase volume by 5%
 */
export async function volumeUp(): Promise<void> {
    await invoke('volume_up');
}

/**
 * Decrease volume by 5%
 */
export async function volumeDown(): Promise<void> {
    await invoke('volume_down');
}

/**
 * Toggle mute
 */
export async function toggleMute(): Promise<void> {
    await invoke('toggle_mute');
}

/**
 * Get current volume level and mute status
 */
export async function getVolume(): Promise<VolumeLevel> {
    return await invoke('get_volume');
}

/**
 * Empty the trash
 * @returns Number of items removed
 */
export async function emptyTrash(): Promise<number> {
    return await invoke('empty_trash');
}

/**
 * Eject a drive
 * @param device Device path (e.g., /dev/sdb1)
 */
export async function ejectDrive(device: string): Promise<void> {
    await invoke('eject_drive', { device });
}
