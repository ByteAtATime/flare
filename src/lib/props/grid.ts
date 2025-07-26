import { ImageLikeSchema } from '@flare/protocol';
import { z } from 'zod/v4';
import { ColorLikeSchema } from './color';

export const GridFitSchema = z.enum(['contain', 'fill']);
export type GridFit = z.infer<typeof GridFitSchema>;

export const GridInsetSchema = z.enum(['small', 'medium', 'large']);
export type GridInset = z.infer<typeof GridInsetSchema>;

export const GridPropsSchema = z.object({
	filtering: z.union([z.boolean(), z.object({ keepSectionOrder: z.boolean() })]).optional(),
	throttle: z.boolean().default(false),
	columns: z.number().default(6),
	searchBarPlaceholder: z.string().optional(),
	onSearchTextChange: z.boolean().optional(),
	onSelectionChange: z.boolean().optional(),
	isLoading: z.boolean().default(false),
	inset: GridInsetSchema.optional(),
	aspectRatio: z.enum(['1', '3/2', '2/3', '4/3', '3/4', '16/9', '9/16']).optional(),
	fit: GridFitSchema.optional(),
	pagination: z
		.object({
			hasMore: z.boolean(),
			onLoadMore: z.boolean(),
			pageSize: z.number()
		})
		.optional(),
	selectedItemId: z.string().optional()
});
export type GridProps = z.infer<typeof GridPropsSchema>;

const GridItemContentValueSchema = z.union([
	ImageLikeSchema,
	z.object({
		color: ColorLikeSchema
	})
]);

export const GridItemContentSchema = z.union([
	GridItemContentValueSchema,
	z.object({
		tooltip: z.string(),
		value: GridItemContentValueSchema
	})
]);
export type GridItemContent = z.infer<typeof GridItemContentSchema>;

export const GridItemAccessorySchema = z.object({
	icon: ImageLikeSchema.optional(),
	text: z.string().optional(),
	tooltip: z.string().optional()
});
export type GridItemAccessory = z.infer<typeof GridItemAccessorySchema>;

export const GridItemPropsSchema = z.object({
	id: z.string().optional(),
	content: GridItemContentSchema,
	title: z.string().optional(),
	subtitle: z.string().optional(),
	keywords: z.array(z.string()).optional(),
	accessory: GridItemAccessorySchema.optional(),
	quickLook: z.object({ name: z.string().optional(), path: z.string() }).optional()
});
export type GridItemProps = z.infer<typeof GridItemPropsSchema>;

export const GridSectionPropsSchema = z.object({
	title: z.string().optional(),
	subtitle: z.string().optional(),
	columns: z.number().optional(),
	aspectRatio: z.enum(['1', '3/2', '2/3', '4/3', '3/4', '16/9', '9/16']).optional(),
	fit: GridFitSchema.optional(),
	inset: GridInsetSchema.optional()
});
export type GridSectionProps = z.infer<typeof GridSectionPropsSchema>;

export const GridEmptyViewPropsSchema = z.object({
	title: z.string(),
	description: z.string().optional(),
	icon: ImageLikeSchema.optional().catch(undefined)
});
export type GridEmptyViewProps = z.infer<typeof GridEmptyViewPropsSchema>;
