import { z } from 'zod/v4';
import { ImageLikeSchema } from '@flare/protocol';
import { ColorLikeSchema } from './color';
import { TextWithColorSchema } from './text';

export const DetailMetadataLabelPropsSchema = z.object({
	title: z.string(),
	text: z.union([z.string(), TextWithColorSchema]).optional(),
	icon: ImageLikeSchema.optional(),
	isLoading: z.boolean().default(false)
});
export type DetailMetadataLabelProps = z.infer<typeof DetailMetadataLabelPropsSchema>;

export const DetailMetadataLinkPropsSchema = z.object({
	title: z.string(),
	text: z.string(),
	target: z.string()
});
export type DetailMetadataLinkProps = z.infer<typeof DetailMetadataLinkPropsSchema>;

export const DetailMetadataTagListItemPropsSchema = z
	.object({
		text: z.string().optional(),
		color: ColorLikeSchema,
		icon: ImageLikeSchema.optional()
	})
	.refine((data) => data.text != null || data.icon != null, {
		message: 'Either text or icon must be provided for TagList.Item'
	});
export type DetailMetadataTagListItemProps = z.infer<typeof DetailMetadataTagListItemPropsSchema>;

export const DetailMetadataTagListPropsSchema = z.object({
	title: z.string()
});
export type DetailMetadataTagListProps = z.infer<typeof DetailMetadataTagListPropsSchema>;

export const DetailMetadataSeparatorPropsSchema = z.object({});
export type DetailMetadataSeparatorProps = z.infer<typeof DetailMetadataSeparatorPropsSchema>;

export const DetailMetadataPropsSchema = z.object({});
export type DetailMetadataProps = z.infer<typeof DetailMetadataPropsSchema>;

export const DetailPropsSchema = z.object({
	isLoading: z.boolean().default(false),
	markdown: z.string().optional(),
	navigationTitle: z.string().optional()
});
export type DetailProps = z.infer<typeof DetailPropsSchema>;
