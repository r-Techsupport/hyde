<script lang="ts">
	import SideBar from './nav/SideBar.svelte';
	import TopBar from './nav/TopBar.svelte';
	import { renderMarkdown } from '$lib/render';

	let editorText = '';
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

	async function fileSelectionHandler(e: CustomEvent) {
		console.log("path: ", e.detail.path)
		const response = await fetch(`http://127.0.0.1:8080/api/doc?path=${encodeURIComponent(e.detail.path)}`);
		// console.log(await response.text());
		const data = await response.json();
		editorText = data.contents;
		renderMarkdown(editorText, previewWindow);
	}

</script>

<div class="container">
	<SideBar on:fileselect={fileSelectionHandler}/>
	<div style="display: flex;flex-direction: column; height: 100vh;">
		<TopBar />
		<div class="editor-controls">
			<!-- Cancel -->
			<svg xmlns="http://www.w3.org/2000/svg" height="40px" viewBox="0 -960 960 960" width="40px">
				<path
					d="M840-680v480q0 33-23.5 56.5T760-120H200q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h480l160 160Zm-80 34L646-760H200v560h560v-446ZM480-240q50 0 85-35t35-85q0-50-35-85t-85-35q-50 0-85 35t-35 85q0 50 35 85t85 35ZM240-560h360v-160H240v160Zm-40-86v446-560 114Z"
				/>
			</svg>
			<!-- Save -->
			<svg xmlns="http://www.w3.org/2000/svg" height="40px" viewBox="0 -960 960 960" width="40px">
				<path
					d="m336-280 144-144 144 144 56-56-144-144 144-144-56-56-144 144-144-144-56 56 144 144-144 144 56 56ZM480-80q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z"
				/>
			</svg>
		</div>
		<div class="editor-panes">
			<textarea bind:value={editorText} class="editor-pane"></textarea>
			<div bind:this={previewWindow} class="preview-pane"></div>
		</div>
	</div>
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
	}

	/* div containing both the preview pane and the editor pane */
	.editor-panes {
		height: 100%;
		/* flex-direction: row;
		flex-grow: 1; */
		/* max-height: 80%;
		height: 80%; */
		overflow-y: scroll;
	}

	.editor-pane {
		resize: none;
		box-sizing: border-box;
		width: calc((100vw - var(--sidebar-width)) / 2);
		height: 98%;
		padding: 1rem;
		border: none;
		font-size: larger;
		background-color: var(--background-0);
		color: var(--foreground-0);
		/* overflow-y: scroll; */
	}

	.editor-pane:focus {
		border: 0px;
	}

	.preview-pane {
		/* sizing and spacing */
		float: right;
		box-sizing: border-box;
		width: calc((100vw - var(--sidebar-width)) / 2 - 5px);
		height: 100%;
		padding-left: 1rem;
		padding-right: 1rem;
		/* inn */
		border-left: 0.07rem solid var(--foreground-5);

		/* styling of rendered text */
		color: var(--foreground-0);
		font-family:
			system-ui,
			-apple-system,
			BlinkMacSystemFont,
			'Segoe UI',
			Roboto,
			Oxygen,
			Ubuntu,
			Cantarell,
			'Open Sans',
			'Helvetica Neue',
			sans-serif;
		overflow-y: scroll;
	}

	.preview-pane :global(a) {
		color: var(--foreground-0),
	}
</style>
