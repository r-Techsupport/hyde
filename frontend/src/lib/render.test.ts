import { describe, test, expect, vi } from 'vitest';
import { renderMarkdown } from './render';
import { addToast } from './toast';

// Mock the addToast function
vi.mock('./toast', async (importOriginal) => {
	const mod = await importOriginal<typeof import('./toast')>();
	return {
		...mod,
		addToast: vi.fn()
	};
});

// Mock dompurify
vi.mock('dompurify', () => ({
	default: {
		sanitize: vi.fn((dirty: string) => dirty),
		removed: []
	}
}));

describe('renderMarkdown', () => {
	test('renders title and description correctly', async () => {
		const input = `---
title: My Title
---
# Heading
Content here.`;

		const mockOutput = { innerHTML: '' } as HTMLElement;

		await renderMarkdown(input, mockOutput);

		expect(mockOutput.innerHTML).toContain('<h1 class="doc-title">My Title</h1>');
		expect(mockOutput.innerHTML).toContain('<h1>Heading</h1>');
		expect(mockOutput.innerHTML).toContain('Content here.');
	});

	test('displays error toast when frontmatter header is missing', () => {
		const input = `---
layout
---
---`;

		const mockOutput = { innerHTML: '' } as HTMLElement;

		renderMarkdown(input, mockOutput);

		// Check that addToast was called at least once with the error message
		expect(addToast).toHaveBeenCalled();
		expect(addToast).toHaveBeenCalledWith(
			'Missing front matter: Ensure the title is defined.',
			expect.anything(),
			false
		);
	});

	test('preserves title and description when frontmatter is malformed', async () => {
		const input = `---
title: My Title
description: My Description
---`;

		const mockOutput = { innerHTML: '' } as HTMLElement;

		await renderMarkdown(input, mockOutput);

		expect(mockOutput.innerHTML).toContain('<h1 class="doc-title">My Title</h1>');
		expect(mockOutput.innerHTML).toContain('<p class="doc-description">My Description</p>');
	});
});
