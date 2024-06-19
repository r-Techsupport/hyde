<script lang="ts">
	import SideBar from './nav/SideBar.svelte';
	import TopBar from './nav/TopBar.svelte';
	import ChangeDialogue from './ChangeDialogue.svelte';
	import { renderMarkdown } from '$lib/render';
	import { cache } from '$lib/cache';
	import { apiAddress } from '$lib/net';
	import LoadingIcon from './LoadingIcon.svelte';
	import { ToastType, addToast } from '$lib/toast';
	import Toasts from './Toasts.svelte';
	import { currentFile } from '$lib/main';
	import { get } from 'svelte/store';

	/** The text currently displayed in the editing window */
	let editorText = '';
	/** A reference to the div where markdown is rendered to */
	let previewWindow: HTMLElement;
	/** The width of the sidebar */
	export let sidebarWidth = '14rem';
	$: sidebarWidth;
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
	let showLoadingIcon: boolean;

	async function fileSelectionHandler(e: CustomEvent) {
		// If the file in cache doesn't differ from the editor or no file is selected, there are no unsaved changes
		if ((await cache.get(get(currentFile))) === editorText || get(currentFile) === '') {
			currentFile.set(e.detail.path);
			editorText =
				(await cache.get(e.detail.path)) ??
				'Something went wrong, the file tree reported by the backend references a nonexistent file.';
			renderMarkdown(editorText, previewWindow);
		} else if (e.detail.path === currentFile) {
			// Do nothing
		} else {
			// Unsaved changes
			showChangeDialogue = true;
		}
	}

	async function saveChangesHandler() {
		showLoadingIcon = true;
		let response = await fetch(`${apiAddress}/api/doc`, {
			method: 'PUT',
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				contents: editorText,
				path: currentFile
			})
		});
		showLoadingIcon = false;
		switch (response.status) {
			case 201:
				// TODO: Show created message, flush cache
				addToast({
					message: 'Changes synced successfully.',
					type: ToastType.Success,
					dismissible: true,
					timeout: 3000
				});
				break;
			default:
				addToast({
					message: `An error was encountered syncing changes, please report to the developer (Code ${response.status}: "${response.statusText}").`,
					type: ToastType.Error,
					dismissible: true
				});
			// TODO: show error message

			// At some point the editor should make sure it's got a valid token
			// when you first open it, so we don't need to worry about 401 or 403
		}
	}

</script>

<div style="--sidebar-width: {sidebarWidth}" class="container">
	<Toasts />
	<SideBar on:fileselect={fileSelectionHandler} bind:sidebarWidth />
	<div style="display: flex; flex-direction: column; height: 100vh;">
		<TopBar />
		<div class="editor-controls">
			<!-- Cancel -->
			<svg
				xmlns="http://www.w3.org/2000/svg"
				height="40px"
				viewBox="0 -960 960 960"
				width="40px"
				class="cancel"
			>
				<title>Cancel Changes</title>
				<path
					d="m336-280 144-144 144 144 56-56-144-144 144-144-56-56-144 144-144-144-56 56 144 144-144 144 56 56ZM480-80q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z"
				/>
			</svg>
			<!-- Save -->
			<!-- svelte-ignore a11y-click-events-have-key-events -->
			<svg
				on:click={saveChangesHandler}
				role="button"
				tabindex="0"
				xmlns="http://www.w3.org/2000/svg"
				height="40px"
				viewBox="0 -960 960 960"
				width="40px"
				class="publish"
			>
				<title>Publish Changes</title>
				<path d="M382-240 154-468l57-57 171 171 367-367 57 57-424 424Z" />
			</svg>
		</div>
		<div class="editor-panes">
			<textarea bind:value={editorText} class="editor-pane"></textarea>
			<div bind:this={previewWindow} class="preview-pane"></div>
		</div>
	</div>
	<LoadingIcon bind:visible={showLoadingIcon} />
	<ChangeDialogue bind:visible={showChangeDialogue} />
</div>

<svelte:window on:keydown={onKeyDown} />

<style>
	.container {
		/* --sidebar-width: 14rem; */
		background-color: var(--background-0);
		display: flex;
	}

	.editor-controls {
		border-radius: 5%;
		padding-right: 0.5rem;
		margin-top: 0.2rem;
		border-bottom: 0.07rem solid;
		border-color: var(--foreground-5);
	}

	.editor-controls * {
		border-radius: 5%;
		fill: var(--foreground-5);
		float: right;
		flex-direction: vertical;
		margin: 0.3rem;
		cursor: pointer;
	}

	.publish:hover {
		background-color: var(--background-0);
		box-sizing: border-box;
		border: 0.2rem var(--green) solid;
		transition: all 0.05s ease-out;
	}

	.publish:hover > * {
		fill: var(--green);
	}

	.publish:active {
		border-radius: 5%;
		background-color: var(--green);
	}

	.publish:active > * {
		fill: var(--background-0);
	}

	.cancel:hover {
		background-color: var(--background-0);
		box-sizing: border-box;
		border: 0.2rem var(--red) solid;
		transition: all 0.05s ease-out;
	}

	.cancel:hover > * {
		fill: var(--red);
	}

	.cancel:active {
		background-color: var(--red);
	}

	.cancel:active > * {
		fill: var(--background-0);
	}

	/* div containing both the preview pane and the editor pane */
	.editor-panes {
		height: 100%;
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
