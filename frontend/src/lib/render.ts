/**
 * @file
 * This file contains code used for rendering markdown code
 */
import { marked, type TokensList } from 'marked';
import DOMPurify from 'dompurify';

/**
 * Compile the provided input string into markdown and render it,
 * editing the provided html element
 * @param input The raw markdown to be rendered
 * @param output The element to insert the markdown into
 */
export async function renderMarkdown(input: string, output: InnerHTML): Promise<undefined> {
	// https://marked.js.org/#demo

	let rawTokens: TokensList = marked.lexer(input);

	// Remove FrontMatter

	rawTokens.shift();
	let frontMatter: string = '';
	try {
		frontMatter = rawTokens.shift()['raw'].replace('---\n', '\n').trim();
		// frontMatterPrint = `<details><summary>Show FrontMatter</summary><pre>` + frontMatter + `</pre></details>`;
		// console.log(frontMatter);
	} catch {
		console.log('No FrontMatter found!');
	}

	const cleanedOutput: string = DOMPurify.sanitize(await marked.parser(rawTokens));
	if (DOMPurify.removed.length > 0) {
		console.warn('Possible XSS detected, modified output: ', DOMPurify.removed);
	}

	output.innerHTML = cleanedOutput;
}
