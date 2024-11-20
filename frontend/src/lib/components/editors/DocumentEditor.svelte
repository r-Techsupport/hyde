<script lang="ts">
	import { onDestroy } from 'svelte';
	import { currentFile, editorText } from '$lib/main';
	import { addToast, ToastType } from '$lib/toast';
	import { get } from 'svelte/store';
	import { cache } from '$lib/cache';
	import { renderMarkdown } from '$lib/render';

	const charCount = 500;

	let showCommitModal = $state(false);
	let commitModal: HTMLElement = $state();
	let commitMessageInput: string = $state('');

	let previousFile: string | null = null;

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
				renderMarkdown($editorText, previewWindow);
			}
		}, DEBOUNCE_TIME);
	}

	/**
	 * Check and see if any changes were made to
	 * the currently selected document
	 */
	async function hasChanges(): Promise<boolean> {
		const storedText = await cache.get(get(currentFile));
		return $editorText !== (storedText ?? '');
	}

	async function confirmCommitHandler() {
		const commitMessage = commitMessageInput.trim();

		if (!(await hasChanges())) {
			addToast({
				message: `No changes detected to commit.`,
				type: ToastType.Error,
				timeout: 1000,
				dismissible: true
			});
			return;
		}

		showCommitModal = false;
		await saveChangesHandler(commitMessage);
	}

	interface Props {
		previewWindow: HTMLElement;
		saveChangesHandler: (commitMessage: string) => Promise<void>;
		createPullRequestHandler: () => Promise<void>;
	}

	let { previewWindow = $bindable(), saveChangesHandler, createPullRequestHandler }: Props = $props();

	async function cancelChangesHandler() {
		if ($editorText !== get(currentFile)) {
			$editorText =
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
		if (v !== '') {
			$editorText = (await cache.get(v)) ?? '';
		}
	});

	const unsubscribe = currentFile.subscribe(async (file) => {
		if (file !== previousFile && showCommitModal) {
			showCommitModal = false;
		}
		previousFile = file;
		$editorText = (await cache.get(file)) ?? '';
	});

	onDestroy(() => {
		unsubscribe();
	});
</script>

<div class="editor-controls">
	<!-- Cancel -->
	<button onclick={cancelChangesHandler} class="cancel" title="Cancel Changes">
		<span>Cancel Changes</span>
		<svg xmlns="http://www.w3.org/2000/svg" height="40px" viewBox="0 -960 960 960" width="40px">
			<title>Cancel Changes</title>
			<path
				d="m336-280 144-144 144 144 56-56-144-144 144-144-56-56-144 144-144-144-56 56 144 144-144 144 56 56ZM480-80q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z"
			/>
		</svg>
	</button>
	<!-- Save -->
	<button
		onclick={async () => {
			showCommitModal = true;
		}}
		class="publish"
		title="Publish Changes"
	>
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
	<!-- Pull Request -->
	<button onclick={createPullRequestHandler} class="pull-request" title="Create Pull Request">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="25px"
			height="25px"
			viewBox="0 0 24 24"
			fill="none"
		>
			<path
				fill-rule="evenodd"
				clip-rule="evenodd"
				d="M14.7071 2.70711L13.4142 4H14C17.3137 4 20 6.68629 20 10V16.1707C21.1652 16.5825 22 17.6938 22 19C22 20.6569 20.6569 22 19 22C17.3431 22 16 20.6569 16 19C16 17.6938 16.8348 16.5825 18 16.1707V10C18 7.79086 16.2091 6 14 6H13.4142L14.7071 7.29289C15.0976 7.68342 15.0976 8.31658 14.7071 8.70711C14.3166 9.09763 13.6834 9.09763 13.2929 8.70711L10.2929 5.70711C9.90237 5.31658 9.90237 4.68342 10.2929 4.29289L13.2929 1.29289C13.6834 0.902369 14.3166 0.902369 14.7071 1.29289C15.0976 1.68342 15.0976 2.31658 14.7071 2.70711ZM18 19C18 18.4477 18.4477 18 19 18C19.5523 18 20 18.4477 20 19C20 19.5523 19.5523 20 19 20C18.4477 20 18 19.5523 18 19ZM6 4C5.44772 4 5 4.44772 5 5C5 5.55228 5.44772 6 6 6C6.55228 6 7 5.55228 7 5C7 4.44772 6.55228 4 6 4ZM7 7.82929C8.16519 7.41746 9 6.30622 9 5C9 3.34315 7.65685 2 6 2C4.34315 2 3 3.34315 3 5C3 6.30622 3.83481 7.41746 5 7.82929V16.1707C3.83481 16.5825 3 17.6938 3 19C3 20.6569 4.34315 22 6 22C7.65685 22 9 20.6569 9 19C9 17.6938 8.16519 16.5825 7 16.1707V7.82929ZM6 18C5.44772 18 5 18.4477 5 19C5 19.5523 5.44772 20 6 20C6.55228 20 7 19.5523 7 19C7 18.4477 6.55228 18 6 18Z"
				fill="#000000"
			/>
		</svg>
		<span>Create Pull Request</span>
	</button>
</div>
<div class="editor-panes">
	<textarea bind:value={$editorText} class="editor-pane"></textarea>
	<div bind:this={previewWindow} class="preview-pane"></div>
</div>

{#if showCommitModal}
	<div
		onclick={() => {
			showCommitModal = false;
		}}
		onkeydown={(e) => {
			if (e.key === 'Escape') {
				showCommitModal = false;
			}
		}}
		role="button"
		tabindex="0"
		class="commit-modal-backdrop"
	></div>
	<div class="commit-modal" bind:this={commitModal}>
		<div class="commit-modal-content">
			<svg
				onclick={() => {
					showCommitModal = false;
				}}
				onkeypress={() => {
					showCommitModal = false;
				}}
				class="commit-modal-close"
				role="button"
				tabindex="0"
				xmlns="http://www.w3.org/2000/svg"
				height="24px"
				viewBox="0 -960 960 960"
				width="24px"
				fill="#e8eaed"
			>
				<path
					d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"
				/>
			</svg>
			<h2>Confirm changes before committing:</h2>
			<h5>Enter a commit message (optional)</h5>
			<input
				type="text"
				placeholder="Enter your commit message here"
				bind:value={commitMessageInput}
				maxlength={charCount}
			/>
			<p class="commit-modal-chars-remaining">
				{charCount - commitMessageInput.length} characters remaining
			</p>
			<div class="commit-modal-buttons">
				<button onclick={() => (showCommitModal = false)}>Deny</button>
				<button onclick={confirmCommitHandler}>Confirm</button>
			</div>
		</div>
	</div>
{/if}

<svelte:window onkeydown={onKeyDown} />

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

	.pull-request {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0.125rem 0.25rem;
	}

	.pull-request span {
		margin-left: 0.25rem;
	}

	.pull-request svg {
		margin-top: 0.125rem;
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

	.commit-modal-backdrop {
		position: absolute;
		top: 0;
		left: 0;
		background-color: var(--background-0);
		opacity: 0.9;
	}

	.commit-modal {
		position: fixed;
		top: 0;
		display: flex;
		align-self: center;
		justify-content: center;
		z-index: 1;
		margin-top: 6rem;
		width: 50%;
		height: 12rem;
	}

	.commit-modal-content {
		margin: auto;
		padding: 1rem;
		width: 90%;
		flex-shrink: 0;

		/* Appearance */
		border: 1px solid var(--background-2);
		border-radius: 5px;
		background-color: var(--background-1);
		color: var(--foreground-0);
		font-family: var(--font-family);
	}

	.commit-modal-content h2 {
		margin: 0;
		margin-bottom: 0.5rem;
	}

	.commit-modal-content input {
		margin-bottom: 0.5rem;
		padding-left: 0.5rem;
		width: 98%;
		height: 4rem;
		background-color: transparent;
		color: var(--foreground-0);
		border-radius: 4px;
		border: 1px solid;
		border-color: var(--foreground-1);
		font-family: var(--font-family);
	}

	.commit-modal-close {
		position: sticky;
		cursor: pointer;
		margin-top: 0.2rem;
		margin-right: 0.2rem;
		float: right;
	}

	.commit-modal-chars-remaining {
		margin: 0;
		padding-left: 0.1rem;
		color: var(--foreground-4);
		font-size: small;
	}

	.commit-modal-buttons {
		display: flex;
		justify-content: flex-end;
		align-items: flex-end;
		gap: 0.2rem;
	}

	.commit-modal-buttons button {
		display: flex;
		justify-content: flex-end;
		align-items: flex-end;
		gap: 0.2rem;
		cursor: pointer;
		height: 2rem;
		background-color: transparent;
		font-size: medium;
		padding: 0.3rem;
		margin: 0.1rem;
		color: var(--foreground-2);
		border-radius: 4px;
		border: 1px solid var(--foreground-4);
		font-family: var(--font-family);
	}

	.commit-modal-buttons button:hover {
		background-color: var(--foreground-4);
		color: var(--background-2);
	}
</style>
