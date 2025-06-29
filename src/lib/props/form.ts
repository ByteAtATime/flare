import { z } from 'zod/v4';

export const FormPropsSchema = z.object({
	enableDrafts: z.boolean().optional(),
	navigationTitle: z.string().optional(),
	isLoading: z.boolean().default(false)
});
export type FormProps = z.infer<typeof FormPropsSchema>;

export const FormTextFieldPropsSchema = z.object({
	id: z.string(),
	autoFocus: z.boolean().default(false),
	defaultValue: z.string().optional(),
	error: z.string().optional(),
	info: z.string().optional(),
	placeholder: z.string().optional(),
	storeValue: z.boolean().default(false),
	title: z.string().optional(),
	value: z.string().optional()
});
export type FormTextFieldProps = z.infer<typeof FormTextFieldPropsSchema>;

export const FormTextAreaPropsSchema = z.object({
	id: z.string(),
	autoFocus: z.boolean().default(false),
	defaultValue: z.string().optional(),
	enableMarkdown: z.boolean().default(false),
	error: z.string().optional(),
	info: z.string().optional(),
	placeholder: z.string().optional(),
	storeValue: z.boolean().default(false),
	title: z.string().optional(),
	value: z.string().optional()
});
export type FormTextAreaProps = z.infer<typeof FormTextAreaPropsSchema>;

export const FormDescriptionPropsSchema = z.object({
	title: z.string().optional(),
	text: z.string()
});
export type FormDescriptionProps = z.infer<typeof FormDescriptionPropsSchema>;
