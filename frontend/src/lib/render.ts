/**
 * @file
 * This file contains code used for rendering markdown code
 */
import { marked, type TokensList } from 'marked';
import DOMPurify from 'dompurify';
import { ToastType, addToast, dismissToast } from './toast';

let toastId = -1;
/**
 * Compile the provided input string into markdown and render it,
 * editing the provided html element
 * @param input The raw markdown to be rendered
 * @param output The element to insert the markdown into
 */
export async function renderMarkdown(input: string, output: InnerHTML): Promise<undefined> {
	// https://marked.js.org/#demo

	const rawTokens: TokensList = marked.lexer(input);
	stripFrontMatter(rawTokens);
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
		frontMatterNode.type === 'heading' &&
		frontMatterNode.raw.startsWith('layout')
	) {
		// The output of this process will contain the serialized frontmatter header
		frontMatterNode['raw'].replace('---\n', '\n').trim();
		// Hide the toast if a header was detected and it's being displayed
		if (toastId !== -1) {
			dismissToast(toastId);
			toastId = -1;
		}
	} else {
		// -1 means the toast isn't displayed
		if (toastId === -1) {
			toastId = addToast({
				message:
					'No valid frontmatter header was found, please ensure all documents have a frontmatter header',
				type: ToastType.Error,
				dismissible: false
			});
		}

		console.warn('No frontmatter header found');
	}
}
