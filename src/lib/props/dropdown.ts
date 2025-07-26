import { z } from 'zod/v4';
import { ImageLikeSchema } from '@flare/protocol';

export const DropdownItemPropsSchema = z.object({
	value: z.string(),
	title: z.string(),
	keywords: z.array(z.string()).optional(),
	icon: ImageLikeSchema.optional()
});
export type DropdownItemProps = z.infer<typeof DropdownItemPropsSchema>;

export const DropdownSectionPropsSchema = z.object({
	title: z.string().optional()
});
export type DropdownSectionProps = z.infer<typeof DropdownSectionPropsSchema>;

export const DropdownPropsSchema = z.object({
	id: z.string().optional(),
	tooltip: z.string().optional(),
	defaultValue: z.string().optional(),
	filtering: z.union([z.boolean(), z.object({ keepSectionOrder: z.boolean() })]).optional(),
	isLoading: z.boolean().optional(),
	placeholder: z.string().optional(),
	storeValue: z.boolean().optional(),
	throttle: z.boolean().optional(),
	value: z.string().optional().nullable(),
	autoFocus: z.boolean().default(false),
	error: z.string().optional(),
	info: z.string().optional(),
	title: z.string().optional()
});
export type DropdownProps = z.infer<typeof DropdownPropsSchema>;
