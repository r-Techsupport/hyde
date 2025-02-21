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

// The error toast is only displayed when there's a file selected, and we check for issues based on that file selection
vi.mock('./main', async (importOriginal) => {
	const mod = await importOriginal<typeof import('./main')>();
	// A mock writeable store
	const currentFile = {
		subscribe: (func: (m: string) => undefined) => {
			func('foo/bar');
			// The code that accesses this usually uses `get()`, which calls subscribe, then calls `unsubscribe`,
			// which is the function returned by a `subscribe` call
			// https://svelte.dev/docs/svelte-store#writable
			return () => {};
		}
	};
	return {
		...mod,
		currentFile
	};
});

describe('Frontmatter removal robustness', () => {
	test.concurrent('Basic (av-removal header)', async () => {
		const leftover = stripFrontMatterFromString(
			String.raw`---
title: List of AV removers
description: A list of links for downloading (or usage guides of) dedicated uninstallers for 3rd party AVs. 
sidebar:
    hidden: false
has_children: false
parent: Factoids
pagefind: true
last_modified_date: 2024-01-02
---
Below is...`
		);
		assert(leftover === 'Below is...');
	});

	test.concurrent('Basic newline (force-updating-windows header)', async () => {
		const leftover = stripFrontMatterFromString(
			String.raw`---
title: Force updating Windows
sidebar:
    hidden: false
parent: Factoids
has_children: false
pagefind: true
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
