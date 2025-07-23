/**
 * @file
 * Utilities for rendering text into markdown
 */
import fm from 'front-matter';
import { Renderer, marked, type TokensList } from 'marked';
import DOMPurify from 'dompurify';
import { ToastType, addToast, dismissToast } from './toast';
import { apiAddress } from './main';

/**
 * When the rendered file is missing a valid frontmatter header, then an error toast is displayed.
 * If the toast is not displayed, this is set to zero. If it *is* displayed, this is the ID of the toast being rendered.
 */
let toastId = -1;

interface FrontMatter {
	title?: string;
	description?: string;
	[key: string]: unknown;
}

/**
 * Compile the provided input string into markdown and render it,
 * editing the provided html element
 * @param input The raw markdown to be rendered
 * @param output The element to insert the markdown into
 */
export async function renderMarkdown(input: string, output: HTMLElement): Promise<void> {
	// Parse front matter and get title, description, and markdown content
	getFrontMatterType(input);
	const parsed = fm(input);
	const frontMatter = parsed.attributes as FrontMatter;
	const title = frontMatter.title;
	const description = frontMatter.description;
	const content = parsed.body;

	checkFrontMatter(title);

	// Convert content to tokens and process images
	marked.use({ renderer: new Renderer() });
	const rawTokens: TokensList = marked.lexer(content);
	marked.walkTokens(rawTokens, (t) => {
		if (t.type === 'image') {
			t.href = t.href.replace(/^(?:\.\.\/)+/, '/');
			if (t.href.startsWith('/')) {
				t.href = apiAddress + t.href;
			}
		}
	});

	// Generate sanitized HTML body
	const bodyHtml = DOMPurify.sanitize(await marked.parser(rawTokens));
	if (DOMPurify.removed.length > 0) {
		console.warn('Possible XSS detected, modified output:', DOMPurify.removed);
	}

	// Prepend title and description as <h1> and <p> if defined
	let outputHtml = '';
	if (title) {
		outputHtml += `<h1 class="doc-title">${DOMPurify.sanitize(title)}</h1>\n`;
	}
	if (description) {
		outputHtml += `<p class="doc-description">${DOMPurify.sanitize(description)}</p>\n`;
	}
	outputHtml += bodyHtml;

	output.innerHTML = outputHtml;
}

export function checkFrontMatter(title?: string): void {
	if (!title) {
		// Display a toast notification if title is missing
		if (toastId === -1) {
			toastId = addToast(
				'Missing front matter: Ensure the title is defined.',
				ToastType.Error,
				false
			);
		}
	} else {
		// Hide the toast if title is present
		if (toastId !== -1) {
			dismissToast(toastId);
			toastId = -1;
		}
	}
}

export function getFrontMatterType(input: string): 'yaml' | 'toml' | 'json' | 'unknown' {
	const trimmed = input.trim();

	if (trimmed.startsWith('---') && trimmed.endsWith('---')) {
		return 'yaml';
	} else if (trimmed.startsWith('+++') && trimmed.endsWith('+++')) {
		return 'toml';
	} else if (trimmed.startsWith('{') && trimmed.endsWith('}')) {
		return 'json';
	}

	// Display a toast notification if front matter is not YAML
	if (toastId === -1) {
		toastId = addToast(
			'Warning: Front matter is not in YAML format. YAML is recommended.',
			ToastType.Warning,
			false
		);
	}

	return 'unknown';
}
