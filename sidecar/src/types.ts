import type React from 'react';
import type Reconciler from 'react-reconciler';

export type ComponentType = string | React.ComponentType;
export type ComponentProps = Record<string, unknown>;

export interface BaseInstance {
	id: number;
	_internalFiber?: Reconciler.Fiber;
}

export interface FlareInstance extends BaseInstance {
	type: ComponentType;
	props: ComponentProps;
	_unserializedProps?: ComponentProps;
	children: (FlareInstance | TextInstance)[];
	namedChildren?: { [key: string]: number };
}

export interface TextInstance extends BaseInstance {
	type: 'TEXT';
	text: string;
}

export interface Container {
	id: 'root';
	children: (FlareInstance | TextInstance)[];
}

export type AnyInstance = FlareInstance | TextInstance;
export type ParentInstance = FlareInstance | Container;
export type UpdatePayload = Record<string, unknown>;

export interface SerializedReactElement {
	$$typeof: 'react.element.serialized';
	type: string;
	props: Record<string, unknown>;
}

export enum ToastStyle {
	Success = 'SUCCESS',
	Failure = 'FAILURE',
	Animated = 'ANIMATED'
}

export interface ToastActionOptions {
	title: string;
	onAction?: (toast: Toast) => void;
	shortcut?: { modifiers: string[]; key: string };
}

export interface Toast {
	id: number;
	title: string;
	message?: string;
	style?: ToastStyle;
	primaryAction?: ToastActionOptions;
	secondaryAction?: ToastActionOptions;
	hide(): Promise<void>;
	show(): Promise<void>;
}
