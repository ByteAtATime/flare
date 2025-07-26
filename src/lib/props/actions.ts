import { z } from 'zod/v4';
import { ImageLikeSchema } from '@flare/protocol';
import { platform } from '@tauri-apps/plugin-os';

const KeyModifierSchema = z.enum(['cmd', 'ctrl', 'opt', 'shift']);
const KeyEquivalentSchema = z.string();

export const KeyboardShortcutSchema = z.object({
	modifiers: z.array(KeyModifierSchema),
	key: KeyEquivalentSchema
});
export type KeyboardShortcut = z.infer<typeof KeyboardShortcutSchema>;

export const keyEventMatches = (event: KeyboardEvent, shortcut: KeyboardShortcut) => {
	if (event.key.toLowerCase() !== shortcut.key.toLowerCase()) {
		return false;
	}

	const isMac = platform() === 'macos';
	const required = new Set(shortcut.modifiers);

	const metaExpected = isMac ? required.has('cmd') : false;
	const ctrlExpected = isMac ? required.has('ctrl') : required.has('cmd') || required.has('ctrl');
	const altExpected = required.has('opt');
	const shiftExpected = required.has('shift');

	return (
		event.metaKey === metaExpected &&
		event.ctrlKey === ctrlExpected &&
		event.altKey === altExpected &&
		event.shiftKey === shiftExpected
	);
};

const ActionStyleSchema = z.enum(['regular', 'destructive']);

export const ActionPropsSchema = z.object({
	title: z.string(),
	icon: ImageLikeSchema.optional(),
	shortcut: KeyboardShortcutSchema.optional(),
	style: ActionStyleSchema.optional()
});
export type ActionProps = z.infer<typeof ActionPropsSchema>;

export const ActionPushPropsSchema = z.object({
	title: z.string(),
	icon: ImageLikeSchema.optional(),
	shortcut: KeyboardShortcutSchema.optional()
});

export const ActionCopyToClipboardPropsSchema = z.object({
	content: z.string(),
	title: z.string().optional(),
	icon: ImageLikeSchema.optional(),
	shortcut: KeyboardShortcutSchema.optional()
});
export type ActionCopyToClipboardProps = z.infer<typeof ActionCopyToClipboardPropsSchema>;

export const ActionOpenInBrowserPropsSchema = z.object({
	url: z.url(),
	title: z.string().optional(),
	icon: ImageLikeSchema.optional(),
	shortcut: KeyboardShortcutSchema.optional()
});
export type ActionOpenInBrowserProps = z.infer<typeof ActionOpenInBrowserPropsSchema>;

export const ActionSubmitFormPropsSchema = z.object({
	title: z.string().optional(),
	icon: ImageLikeSchema.optional(),
	shortcut: KeyboardShortcutSchema.optional(),
	style: ActionStyleSchema.optional()
});
export type ActionSubmitFormProps = z.infer<typeof ActionSubmitFormPropsSchema>;

export const ActionPanelSectionPropsSchema = z.object({
	title: z.string().optional()
});
export type ActionPanelSectionProps = z.infer<typeof ActionPanelSectionPropsSchema>;

export const ActionPanelPropsSchema = z.object({});
export type ActionPanelProps = z.infer<typeof ActionPanelPropsSchema>;
