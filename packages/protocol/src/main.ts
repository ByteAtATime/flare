import { z } from 'zod/v4';
import {
	BrowserExtensionRequestMessageSchema,
	InvokeCommandMessageSchema,
	OauthAuthorizeMessageSchema,
	OauthGetTokensMessageSchema,
	OauthRemoveTokensMessageSchema,
	OauthSetTokensMessageSchema,
	OpenMessageSchema
} from './api';
import {
	AiAskStreamMessageSchema,
	AiCanAccessMessageSchema,
	AiStreamChunkMessageSchema,
	AiStreamEndMessageSchema,
	AiStreamErrorMessageSchema
} from './ai';
import { CommandSchema } from './command';
import { ShowHudMessageSchema } from './hud';
import { GoBackToPluginListSchema, PluginListSchema, PreferenceValuesSchema } from './plugin';

export const BatchUpdateSchema = z.object({
	type: z.literal('BATCH_UPDATE'),
	payload: z.array(CommandSchema)
});
export type BatchUpdate = z.infer<typeof BatchUpdateSchema>;

const LogMessageSchema = z.object({
	type: z.literal('log'),
	payload: z.unknown()
});

const FocusElementMessageSchema = z.object({
	type: z.literal('FOCUS_ELEMENT'),
	payload: z.object({ elementId: z.number() })
});

const ResetElementMessageSchema = z.object({
	type: z.literal('RESET_ELEMENT'),
	payload: z.object({ elementId: z.number() })
});

export const SidecarMessageSchema = z.union([BatchUpdateSchema, CommandSchema, LogMessageSchema]);
export type SidecarMessage = z.infer<typeof SidecarMessageSchema>;

const CloseMainWindowMessageSchema = z.object({
	type: z.literal('close-main-window'),
	payload: z.object({})
});

export const SidecarMessageWithPluginsSchema = z
	.union([
		BatchUpdateSchema,
		CommandSchema,
		ShowHudMessageSchema,
		LogMessageSchema,
		PluginListSchema,
		PreferenceValuesSchema,
		GoBackToPluginListSchema,
		OpenMessageSchema,
		InvokeCommandMessageSchema,
		BrowserExtensionRequestMessageSchema,
		OauthAuthorizeMessageSchema,
		OauthGetTokensMessageSchema,
		OauthSetTokensMessageSchema,
		OauthRemoveTokensMessageSchema,
		AiAskStreamMessageSchema,
		AiStreamChunkMessageSchema,
		AiStreamEndMessageSchema,
		AiStreamErrorMessageSchema,
		AiCanAccessMessageSchema,
		FocusElementMessageSchema,
		ResetElementMessageSchema,
		CloseMainWindowMessageSchema
	])
	.and(z.object({ timestamp: z.number() }));
export type SidecarMessageWithPlugins = z.infer<typeof SidecarMessageWithPluginsSchema>;
