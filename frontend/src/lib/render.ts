/**
 * @file
 * Utilities for rendering text into markdown
 */
import { Renderer, marked, type TokensList } from 'marked';
import DOMPurify from 'dompurify';
import { ToastType, addToast, dismissToast } from './toast';
import { dev } from '$app/environment';
import { get } from 'svelte/store';
import { currentFile } from './main';

/**
 * When the rendered file is missing a valid frontmatter header, then an error toast is displayed.
 * If the toast is not displayed, this is set to zero. If it *is* displayed, this is the ID of the toast being rendered.
 */
let toastId = -1;

/**
 * Compile the provided input string into markdown and render it,
 * editing the provided html element
 * @param input The raw markdown to be rendered
 * @param output The element to insert the markdown into
 */
export async function renderMarkdown(input: string, output: HTMLElement): Promise<undefined> {
	// https://marked.js.org/#demo
	// This whole pipeline needs to be manually defined otherwise everything breaks
	marked.use({ renderer: new Renderer() });
	const rawTokens: TokensList = marked.lexer(input);
	stripFrontMatter(rawTokens);
	// rewrite image urls to point to the correct location
	if (dev) {
		marked.walkTokens(rawTokens, (t) => {
			if (t.type !== 'image') {
				return;
			}
			if (t.href.startsWith('/')) {
				t.href = 'http://localhost:8080' + t.href;
			}
		});
	}
	const cleanedOutput: string = DOMPurify.sanitize(await marked.parser(rawTokens));
	if (DOMPurify.removed.length > 0) {
		console.warn('Possible XSS detected, modified output: ', DOMPurify.removed);
	}

	output.innerHTML = cleanedOutput;
}

/**
 * Strip the Frontmatter header from the provided list of tokens, returning the modified tree.
 *
 * exported for tests
 */
export function stripFrontMatter(input: TokensList) {
	// Remove the first line break
	input.shift();
	const frontMatterNode = input.shift();
	if (
		frontMatterNode !== undefined &&
		frontMatterNode.type === 'paragraph' &&
		frontMatterNode.raw.includes('title: ')
	) {
		input.shift();
		// Hide the toast if a header was detected and it's being displayed
		if (toastId !== -1) {
			dismissToast(toastId);
			toastId = -1;
		}
	} else {
		if (get(currentFile) !== '') {
			// -1 means the toast isn't displayed
			if (toastId === -1) {
				toastId = addToast(
					'No valid frontmatter header was found, please ensure all documents have a frontmatter header',
					ToastType.Error,
					false
				);
			}
		}
	}
}
