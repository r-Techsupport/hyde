<script lang="ts">
	import SideBar from './nav/SideBar.svelte';
	import TopBar from './nav/TopBar.svelte';
	import ChangeDialogue from './nav/ChangeDialogue.svelte';
	import { renderMarkdown } from '$lib/render';
	import { cache } from '$lib/cache';

	/** The text currently displayed in the editing window */
	let editorText = '';
	/** The path to the file currently being displayed by the window */
	let currentFile = '';
	/** A reference to the div where markdown is rendered to */
	let previewWindow: InnerHTML;
	/**
	 * The time in milliseconds that must pass after a keypress
	 * before markdown is rendered
	 */
	const DEBOUNCE_TIME: number = 500;
	let lastKeyPressedTime = Date.now();

	/**
	 * This function is called whenever a key is pressed.
	 *
	 * @see https://svelte.dev/repl/162005fa12cc4feb9f668e09260595a7?version=3.24.1
	 */
	async function onKeyDown() {
		lastKeyPressedTime = Date.now();
		setTimeout(() => {
			if (lastKeyPressedTime + DEBOUNCE_TIME >= Date.now()) {
				renderMarkdown(editorText, previewWindow);
			}
		}, DEBOUNCE_TIME);
	}

	let showChangeDialogue: boolean;

	async function fileSelectionHandler(e: CustomEvent) {
		// If the file in cache doesn't differ from the editor or no file is selected, there are no unsaved changes
		if ((await cache.get(currentFile)) === editorText || currentFile === '') {
			currentFile = e.detail.path;
			editorText =
				(await cache.get(e.detail.path)) ??
				'Something went wrong, the file tree reported by the backend references a nonexistent file.';
			renderMarkdown(editorText, previewWindow);
		} else if (e.detail.path === currentFile) {
			// do nothing
		}
		else {
			showChangeDialogue = true;
		}
	}
</script>

<div class="container">
	<SideBar on:fileselect={fileSelectionHandler} />
	<div style="display: flex;flex-direction: column; height: 100vh;">
		<TopBar />
		<div class="editor-controls">
			<!-- Cancel -->
			<svg xmlns="http://www.w3.org/2000/svg" height="40px" viewBox="0 -960 960 960" width="40px">
				<title>Cancel Changes</title>
				<path
					d="m336-280 144-144 144 144 56-56-144-144 144-144-56-56-144 144-144-144-56 56 144 144-144 144 56 56ZM480-80q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z"
				/>
			</svg>
			<!-- Save -->
			<svg xmlns="http://www.w3.org/2000/svg" height="40px" viewBox="0 -960 960 960" width="40px" fill="#e8eaed">
				<title>Publish Changes</title>
				<path d="M382-240 154-468l57-57 171 171 367-367 57 57-424 424Z"/>
			</svg>
		</div>
		<div class="editor-panes">
			<textarea bind:value={editorText} class="editor-pane"></textarea>
			<div bind:this={previewWindow} class="preview-pane"></div>
		</div>
	</div>
	<ChangeDialogue bind:visible={showChangeDialogue} />
</div>

<svelte:window on:keydown={onKeyDown} />

<style>
	.container {
		--sidebar-width: 14rem;
		background-color: var(--background-0);
		display: flex;
	}

	.editor-controls {
		padding-right: 0.5rem;
		margin-top: 0.2rem;
		border-bottom: 0.07rem solid;
		border-color: var(--foreground-5);
	}

	.editor-controls * {
		fill: var(--foreground-5);
		float: right;
		flex-direction: vertical;
		margin: 0.3rem;
		cursor: pointer;
	}

	/* div containing both the preview pane and the editor pane */
	.editor-panes {
		height: 100%;
		/* flex-direction: row;
		flex-grow: 1; */
		/* max-height: 80%;
		height: 80%; */
		overflow-y: hidden;
	}

	.editor-pane {
		resize: none;
		float: left;
		box-sizing: border-box;
		width: calc((100vw - var(--sidebar-width)) / 2);
		height: 98%;
		padding: 1rem;
		border: none;
		font-size: larger;
		background-color: var(--background-0);
		color: var(--foreground-0);
	}

	.editor-pane:focus {
		outline-width: 0px;
	}

	.preview-pane {
		/* sizing and spacing */
		float: left;
		box-sizing: border-box;
		width: calc((100vw - var(--sidebar-width)) / 2);
		height: 100%;
		padding-left: 1rem;
		padding-right: 1rem;
		border-left: 0.07rem solid var(--foreground-5);

		/* styling of rendered text */
		color: var(--foreground-0);
		font-family: var(--font-family);
		overflow-y: scroll;
	}

	.preview-pane :global(*) {
		word-break: normal;
	}

	.preview-pane :global(a) {
		color: var(--foreground-0);
	}
</style>
