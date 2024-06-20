import { assert, describe, test, vi } from 'vitest';
import { stripFrontMatter } from './render';
import { marked, type TokensList } from 'marked';

const mocks = vi.hoisted(() => {
	return {
		addToast: vi.fn()
	};
});

vi.mock('./toast', async (importOriginal) => {
	const mod = await importOriginal<typeof import('./toast')>();
	return {
		...mod,
		addToast: mocks.addToast
	};
});

describe('Frontmatter removal robustness', () => {
	test.concurrent('Basic (av-removal header)', async () => {
		const leftover = stripFrontMatterFromString(
			String.raw`---
layout: default
title: List of AV removers
nav_exclude: false
has_children: false
parent: Factoids
search_exclude: false
last_modified_date: 2024-01-02
---
Below is...`
		);
		assert(leftover === 'Below is...');
	});

	test.concurrent('Basic newline (force-updating-windows header)', async () => {
		const leftover = stripFrontMatterFromString(
			String.raw`---
layout: default
title: Force updating Windows
nav_exclude: false
parent: Factoids
has_children: false
search_exclude: false
last_modified_date: 2024-03-09
---
# Force updating Windows
`
		);
		assert(leftover === '# Force updating Windows\n');
	});

	test.concurrent('Leaves extra line break alone (mock header)', async () => {
		const leftover = stripFrontMatterFromString(
			String.raw`---
layout
---
---
`
		);
		assert(leftover === '---\n');
	});

	test.concurrent('Fails on missing header', async () => {
		// This test ensures the addToast method is called
		stripFrontMatterFromString(
			String.raw`---
    layout
    ---
    ---
    `
		);
		assert(
			mocks.addToast.mock.calls.length === 1,
			`Expected 'addToast.mock.calls.length' to be 1, was instead ${mocks.addToast.mock.calls.length}`
		);
	});
});

/** Run the provided string through the frontmatter removal tooling and then re-serialize it into a string for convenience*/
function stripFrontMatterFromString(input: string): string {
	const mockMarkDown: TokensList = marked.lexer(input);
	stripFrontMatter(mockMarkDown);
	return mockMarkDown.map((m) => m.raw).join('');
}
