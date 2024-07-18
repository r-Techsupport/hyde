<script lang="ts">
	import { currentFile } from '$lib/main';
	import { addToast, ToastType } from '$lib/toast';
	import { get } from 'svelte/store';
	import { cache } from '$lib/cache';

	export let editorText: string;
	export let previewWindow: HTMLElement;

	let commitModal: HTMLElement;
	let commitMessageInput: HTMLInputElement;

	function openCommitModal() {
		if (commitModal) {
			commitModal.style.display = 'block';
			commitMessageInput.value = '';
		}
	};

	function closeCommitModal() {
		if (commitModal) {
			commitModal.style.display = 'none';
		}
	};

	async function confirmCommitHandler() {
		const commitMessage = commitMessageInput.value.trim();
		if (!commitMessage) {
			alert('You need to write something!');
			return;
		}
		closeCommitModal();
		await saveChangesHandler(commitMessage);
	};

	export let saveChangesHandler: (commitMessage: string) => Promise<void>;
	async function cancelChangesHandler() {
		if (editorText !== get(currentFile)) {
			editorText =
				(await cache.get(get(currentFile))) ??
				"The current file doesn't exist in cache, please report to the developer";
			addToast({
				message: `Cancelled edits to "${get(currentFile)}""`,
				type: ToastType.Success,
				timeout: 1000,
				dismissible: true
			});
		} else {
			// TODO: The button should actually be disabled when there are no unsaved changes
			addToast({
				message: `There are no unsaved changes to "${get(currentFile)}""`,
				type: ToastType.Error,
				timeout: 1000,
				dismissible: true
			});
		}
	}

	currentFile.subscribe(async (v) => {
		editorText = (await cache.get(v)) ?? '';
	});
</script>



<div class="editor-controls">
	<!-- Cancel -->
	<button on:click={cancelChangesHandler} class="cancel" title="Cancel Changes">
		<span>Cancel Changes</span>
		<svg xmlns="http://www.w3.org/2000/svg" height="40px" viewBox="0 -960 960 960" width="40px">
			<title>Cancel Changes</title>
			<path
				d="m336-280 144-144 144 144 56-56-144-144 144-144-56-56-144 144-144-144-56 56 144 144-144 144 56 56ZM480-80q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z"
			/>
		</svg>
	</button>
	<!-- Save -->
	<button on:click={openCommitModal} class="publish" title="Publish Changes">
		<span>Publish Changes</span>
		<svg
			role="button"
			tabindex="0"
			xmlns="http://www.w3.org/2000/svg"
			height="40px"
			viewBox="0 -960 960 960"
			width="40px"
		>
			<title>Publish Changes</title>
			<path d="M382-240 154-468l57-57 171 171 367-367 57 57-424 424Z" />
		</svg>
	</button>
</div>
<div class="editor-panes">
	<textarea bind:value={editorText} class="editor-pane"></textarea>
	<div bind:this={previewWindow} class="preview-pane"></div>
</div>

<div id="commitModal" class="modal" bind:this={commitModal}>
	<div class="modal-content">
	    <button	class="close" on:click={closeCommitModal} aria-label="Close">
			&times;
		</button>
	    <h2>Enter Commit Message</h2>
	    <input type="text" id="commitMessage" placeholder="Enter your commit message here" bind:this={commitMessageInput}>
	    <button id="confirmBtn" on:click={confirmCommitHandler}>Confirm</button>
	    <button id="cancelBtn" on:click={closeCommitModal}>Cancel</button>
	</div>
</div>

<style>
	.editor-controls {
		padding-right: 0.5rem;
		margin: 0.2rem, 0;
		border-bottom: 0.07rem var(--foreground-4) solid;
		display: flex;
		justify-content: right;
	}

	.editor-controls * {
		border-radius: 0.2rem;
		fill: var(--foreground-4);
		float: right;
		flex-direction: vertical;
		cursor: pointer;
	}

	.editor-controls button {
		background-color: transparent;
		font-family: var(--font-family);
		font-size: medium;
		border: none;
		padding: 0.3rem;
		margin: 0.1rem;
	}

	.editor-controls span {
		align-content: center;
		height: 100%;
		color: var(--foreground-3);
	}

	.publish:hover,
	.cancel:hover {
		background-color: var(--background-2);
		transition: all 0.05s ease-out;
	}

	.publish:active {
		background-color: var(--green);
	}

	.cancel:active {
		background-color: var(--red);
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
		outline-width: 0;
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

	.preview-pane :global(img) {
		width: 90%;
	}

	.modal {
	display: none;
	position: fixed;
	z-index: 1;
	padding-top: 100px;
	left: 0;
	top: 0;
	width: 100%;
	height: 100%;
	overflow: auto;
	background-color: rgb(0,0,0);
	background-color: rgba(0,0,0,0.4);
	}

	.modal-content {
	background-color: #fefefe;
	margin: auto;
	padding: 20px;
	border: 1px solid #888;
	width: 80%;
	}

	/* Close button */
	.close {
	color: #aaa;
	float: right;
	font-size: 28px;
	font-weight: bold;
	}

	.close:hover,
	.close:focus {
	color: black;
	text-decoration: none;
	cursor: pointer;
	}

</style>
