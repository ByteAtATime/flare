import { type ImageLike } from '@flare/protocol';
import { convertFileSrc } from '@tauri-apps/api/core';
import { mode } from 'mode-watcher';
import path from 'path';
import type { ColorLike } from './props';

// this matches any emoji character (u flag = unicode, \p{Emoji} = any unicode emoji)
const EMOJI_REGEX = /\p{Emoji}/u;
const graphemeSegmenter = new Intl.Segmenter();

const iconIsEmoji = (icon: string) => {
	// explanation: some emojis are made up of multiple characters, so we need to check if the icon is a single emoji
	// first of all, we need to segment the icon into graphemes (characters)
	// if it has a single visual character, and one of the corresponding code points is an emoji, then we consider it to be an emoji
	// i genuinely hate unicode pls send help
	const graphemes = graphemeSegmenter.segment(icon);

	return Array.from(graphemes).length === 1 && EMOJI_REGEX.test(icon);
};

type ImageMask = 'circle' | 'roundedRectangle';

export type ResolvedIcon =
	| { type: 'raycast'; name: string; tintColor?: ColorLike }
	| {
			type: 'image';
			src: string;
			mask?: ImageMask;
			tintColor?: ColorLike;
	  }
	| { type: 'emoji'; emoji: string };

export function resolveIcon(
	icon: ImageLike | undefined | null,
	assetsBasePath: string
): ResolvedIcon | null {
	if (!icon) return null;

	if (typeof icon === 'string') {
		if (iconIsEmoji(icon)) {
			return { type: 'emoji', emoji: icon };
		}

		// TODO: better heuristic?
		if (icon.endsWith('-16')) {
			return { type: 'raycast', name: icon };
		}

		if (icon.startsWith('http') || icon.startsWith('data:') || icon.startsWith('blob:')) {
			return { type: 'image', src: icon };
		}

		if (icon.startsWith('/')) {
			return { type: 'image', src: convertFileSrc(icon) };
		}

		return { type: 'image', src: convertFileSrc(path.join(assetsBasePath, icon)) };
	}

	if (typeof icon === 'object' && 'source' in icon) {
		const source =
			typeof icon.source === 'object'
				? mode.current === 'dark'
					? icon.source.dark
					: icon.source.light
				: icon.source;

		// TODO: better heuristic?
		if (source.endsWith('-16')) {
			return {
				type: 'raycast',
				name: source,
				tintColor:
					// TODO: actually handle adjustContrast
					typeof icon.tintColor === 'object'
						? { ...icon.tintColor, adjustContrast: false }
						: icon.tintColor
			};
		}

		let src: string;
		if (source.startsWith('http') || source.startsWith('data:') || source.startsWith('blob:')) {
			src = source;
		} else if (source.startsWith('/')) {
			src = convertFileSrc(source);
		} else {
			src = convertFileSrc(path.join(assetsBasePath, source));
		}

		return {
			type: 'image',
			src: src,
			mask: icon.mask,
			tintColor:
				typeof icon.tintColor === 'object' && icon.tintColor
					? { adjustContrast: false, ...icon.tintColor }
					: icon.tintColor
		};
	}

	if (typeof icon === 'object' && 'fileIcon' in icon) {
		return { type: 'image', src: convertFileSrc(icon.fileIcon) };
	}

	return null;
}
