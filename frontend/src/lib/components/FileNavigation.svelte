<!-- https://svelte.dev/repl/347b37e18b5d4a65bbacfd097536db02?version=4.2.17 -->
<script lang="ts">
	import { createEventDispatcher, tick } from 'svelte';
	import { currentFile } from '$lib/main';
	import { cache } from '$lib/cache';
	import { get } from 'svelte/store';
	import ConfirmationDialogue from './ConfirmationDialogue.svelte';
	interface INode {
		name: string;
		children: INode[];
	}

	export let name = '';
	export let children: INode[] = [];
	export let indent = 1;
	export let path = name;
	export let siblings: INode[] | undefined = undefined;
	let self: HTMLElement;
	let selected = false;
	let open = false;
	let showOptionsMenu = false;
	let optionsMenu: HTMLDivElement;
	let showNewFileInput = false;
	let newFileInput: HTMLInputElement;
	let showDeleteFileDialogue = false;

	const dispatch = createEventDispatcher();

	function fileClickHandler() {
		// If it's a directory, hide and show children
		if (children.length > 0) {
			open = !open;
			// console.log(`Clicked directory with path: "${path}"`);
		} else {
			dispatch('fileselect', {
				path: path
			});
			// console.log(`Clicked file with path: "${path}"`);
		}
	}

	currentFile.subscribe((p) => {
		if (path === p) {
			selected = true;
		} else {
			selected = false;
		}
	});

	async function createDocumentHandler() {
		showOptionsMenu = false;
		showNewFileInput = true;
		await tick();
		newFileInput.value = '.md';
		newFileInput.setSelectionRange(0, 0);
		newFileInput.focus();
	}

	async function deleteDocumentHandler() {
		showOptionsMenu = false;
		if (get(currentFile) === path) {
			currentFile.set('');
		}
		if (siblings !== undefined) {
			// siblings.filter((n) => n.name !== name);
			const entryToRemove = siblings.findIndex(n => n.name === name);
			console.log(siblings.splice(entryToRemove, 1));
		}
		// TODO: requisite backend work, eg create DELETE
		// handler for documents.
		
		// While a re-render would happen when the directory
		// is closed and re-opened, I nuke the current element here
		// because I don't know how else to make it happen immediately
		self.remove();
		console.log(`Document "${path}" would be deleted`);
	}
</script>
<span bind:this={self} class={'container' + (selected ? ' selected' : '')}>
	<button on:click={fileClickHandler} style="padding-left: {indent}rem" class="entry-button">
		{#if children.length > 0}
			<!-- Rendering if the navigation item is a directory -->
			<!-- The chevron -->
			{#if !open}
				<svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px"
					><path d="M504-480 320-664l56-56 240 240-240 240-56-56 184-184Z" /></svg
				>
			{:else}
				<!-- Rotate if it's closed -->
				<svg
					style="transform: rotate(90deg)"
					xmlns="http://www.w3.org/2000/svg"
					height="24px"
					viewBox="0 -960 960 960"
					width="24px"><path d="M504-480 320-664l56-56 240 240-240 240-56-56 184-184Z" /></svg
				>
			{/if}

			{name}
		{:else}
			<!-- Rendering if the navigation item is a file -->
			{name}
		{/if}
	</button>
	<!-- The options button for add new file et cetera -->
	<button
		on:click={async () => {
			showOptionsMenu = true;
			await tick();
			optionsMenu.focus();
		}}
		class="entry-option-menu"
	>
		<svg xmlns="http://www.w3.org/2000/svg" height="18px" viewBox="0 -960 960 960" width="18px"
			><path
				d="M240-400q-33 0-56.5-23.5T160-480q0-33 23.5-56.5T240-560q33 0 56.5 23.5T320-480q0 33-23.5 56.5T240-400Zm240 0q-33 0-56.5-23.5T400-480q0-33 23.5-56.5T480-560q33 0 56.5 23.5T560-480q0 33-23.5 56.5T480-400Zm240 0q-33 0-56.5-23.5T640-480q0-33 23.5-56.5T720-560q33 0 56.5 23.5T800-480q0 33-23.5 56.5T720-400Z"
			/></svg
		>
	</button>
</span>
{#if showNewFileInput}
	<span>
		<input
			on:keydown={(e) => {
				if (e.key === 'Enter') {
					open = true;
					children = [...children, { name: newFileInput.value, children: [] }];
					showNewFileInput = false;
					const now = new Date(Date.now());
					cache.set(
						path + newFileInput.value,
						`---
layout: default
title: Your Document Title Here
nav_exclude: false
has_children: false
parent: Parent Folder Name Here
search_exclude: false
last_modified_date: ${now.getFullYear()}-${now.getMonth() + 1}-${now.getDate()}
---\n\n`
					);
					currentFile.set(path + newFileInput.value);
					console.log(cache.get(get(currentFile)));
				}
				if (e.key === 'Escape') {
					showNewFileInput = false;
				}
			}}
			on:blur={() => {
				showNewFileInput = false;
			}}
			bind:this={newFileInput}
			class="newfile-input"
			type="text"
		/>
	</span>
{/if}

{#if showOptionsMenu}
	<div
		on:click={() => {
			showOptionsMenu = false;
		}}
		on:keydown={(e) => {
			if (e.key === 'Escape') {
				showOptionsMenu = false;
			}
		}}
		role="button"
		tabindex="0"
		class="options-menu-backdrop"
	></div>
	<div tabindex="-1" bind:this={optionsMenu} class="options-menu">
		{#if children.length > 0}
			<!-- Options for if the entry is a directory -->
			<button on:click={createDocumentHandler} title="Create New Document">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					height="24px"
					viewBox="0 -960 960 960"
					width="24px"
					fill="#e8eaed"><path d="M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z" /></svg
				>
				Create New Document
			</button>
		{:else}
			<!-- Options for if the entry is a file -->
			<button
				on:click={() => {
					showDeleteFileDialogue = true;
				}}
				title="Delete Document"
			>
				<svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px"
					><path
						d="M280-120q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520ZM360-280h80v-360h-80v360Zm160 0h80v-360h-80v360ZM280-720v520-520Z"
					/></svg
				>
				Delete Document
			</button>
		{/if}
	</div>
{/if}

{#if showDeleteFileDialogue}
	<ConfirmationDialogue
		confirmHandler={() => {
			deleteDocumentHandler();
		}}
		bind:visible={showDeleteFileDialogue}
	>
		Are you sure you want to delete the file "{name}"?
	</ConfirmationDialogue>
{/if}

{#if open}
	{#each children as child}
		{#if child.children.length === 0}
			<!-- Treat path like file -->
			<svelte:self
				on:fileselect
				name={child.name}
				children={child.children}
				siblings={children}
				indent={indent + 1.5}
				path={path + child.name}
			/>
		{:else}
			<!-- Treat path like directory -->
			<svelte:self
				on:fileselect
				name={child.name}
				children={child.children}
				siblings={children}
				indent={indent + 1}
				path={path + child.name + '/'}
			/>
		{/if}
	{/each}
{/if}

<style>
	.entry-button {
		display: flex;
		cursor: pointer;
		user-select: none;
		border: none;
		background: none;
		color: var(--foreground-0);
		font-size: inherit;
		align-items: center;

		/* Sizing, spacing */
		width: 98%;
		border-radius: 5px;
		margin-left: 1%;
		margin-top: 0.4rem;
		margin-bottom: 0.4rem;
		padding-top: 0.4rem;
		padding-bottom: 0.4rem;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.entry-button * {
		text-overflow: ellipsis;
		vertical-align: middle;
	}

	.container {
		display: flex;
		border-radius: 5px;
		width: 98%;
		margin: auto;
	}

	.container:hover {
		background-color: var(--background-3);
	}

	.container svg {
		fill: var(--foreground-5);
	}

	.selected {
		background-color: var(--background-3);
		border: none;
		border-left: 3px solid var(--foreground-5);
	}

	.selected:hover {
		background-color: var(--background-4);
	}

	.entry-option-menu {
		cursor: pointer;
		fill: var(--foreground-0);
		background: transparent;
		border: none;
		border-radius: 5px;
		margin-left: auto;
	}

	.entry-option-menu svg {
		fill: transparent;
	}

	.entry-option-menu:hover {
		background: var(--foreground-5);
	}

	.options-menu {
		position: absolute;
		margin: 0.2rem;
		width: 15rem;
		background-color: var(--background-2);
		border-radius: 5px;
	}

	.options-menu button {
		display: flex;
		color: var(--foreground-1);
		align-items: center;
		padding-left: 1rem;
		width: 100%;
		height: 2rem;
		border-radius: 0.5rem;
		background-color: transparent;
		border: none;
	}

	.options-menu svg {
		margin-right: 0.3rem;
		fill: var(--foreground-1);
	}

	.options-menu button:hover {
		background-color: var(--background-3);
		cursor: pointer;
	}

	.container:hover > .entry-option-menu > svg {
		fill: var(--foreground-2);
	}

	.options-menu-backdrop {
		position: absolute;
		top: 0;
		left: 0;
		width: 100vw;
		height: 100vh;
	}

	.newfile-input {
		height: 2rem;
		background-color: var(--background-2);
		border: none;
		color: var(--foreground-0);
		width: 100%;
	}
</style>
