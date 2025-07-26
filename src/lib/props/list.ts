import { z } from 'zod/v4';
import { ImageLikeSchema } from '@flare/protocol';
import { ColorLikeSchema } from './color';
import { DetailMetadataPropsSchema } from './detail';

export const ListPropsSchema = z.object({
	filtering: z.boolean().optional(),
	isShowingDetail: z.boolean().optional(),
	onSearchTextChange: z.boolean().optional(),
	isLoading: z.boolean().default(false)
});
export type ListProps = z.infer<typeof ListPropsSchema>;

const TextWithColorSchema = z.object({
	value: z.string(),
	color: ColorLikeSchema.optional()
});

const TagValueSchema = z.union([z.string(), z.date()]);

const TagWithColorSchema = z.object({
	value: TagValueSchema,
	color: ColorLikeSchema.optional()
});

const ListItemAccessorySchema = z.object({
	text: z.union([z.string(), TextWithColorSchema]).optional().nullable(),
	tag: z.union([TagValueSchema, TagWithColorSchema]).optional().nullable(),
	date: z.date().optional().nullable(),
	icon: ImageLikeSchema.optional().nullable(),
	tooltip: z.string().optional().nullable()
});

export const ListItemPropsSchema = z.object({
	icon: ImageLikeSchema.optional(),
	title: z.string(),
	accessories: z.array(ListItemAccessorySchema).optional(),
	keywords: z.array(z.string()).optional()
});
export type ListItemProps = z.infer<typeof ListItemPropsSchema>;

export const ListItemDetailPropsSchema = z.object({
	isLoading: z.boolean().default(false),
	markdown: z.string().default('')
});
export type ListItemDetailProps = z.infer<typeof ListItemDetailPropsSchema>;

export const ListItemDetailMetadataPropsSchema = DetailMetadataPropsSchema;
export type ListItemDetailMetadataProps = z.infer<typeof ListItemDetailMetadataPropsSchema>;
