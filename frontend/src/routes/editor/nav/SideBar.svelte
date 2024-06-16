<script lang="ts">
	import { apiAddress } from '$lib/net';
	import FileNavigation from './FileNavigation.svelte';
	import { onMount } from 'svelte';
	/** Binding to the css variable determining sidebar width */
	export let sidebarWidth: string;

	let rootNode = {
		name: '',
		children: []
	};
	$: rootNode;
	// const MOCK_DIRECTORY = {
	//     name: "Root",
	//     children: [
	//         {
	//             name: "File 1",
	//             children: [],
	//         },
	//         {
	//             name: "File 2",
	//             children: [],
	//         },
	//         {
	//             name: "Directory 1",
	//             children: [
	//                 {
	//                     name: "File1 in dir 1",
	//                     children: [],
	//                 }
	//             ]
	//         },
	//         {
	//             name: "Directory 2",
	//             children: [
	//                 {
	//                     name: "file 1 in dir 2",
	//                     children: [],

	//                 }
	//             ]
	//         }
	//     ],
	// }
	let draggingWindow = false;

	onMount(async () => {
		const response = await fetch(`${apiAddress}/api/tree`);
		rootNode = await response.json();
	});
</script>

<div class="side-bar">
	<!-- Because FileNavigation renders recursively, -->
	<!-- any "outside" css needs to be done in a separate div -->
	<div class="directory-nav">
		<FileNavigation on:fileselect {...rootNode} />
	</div>
</div>

<div
	on:mousedown={() => {
		draggingWindow = true;
	}}
	on:mouseup={() => {
		draggingWindow = false;
	}}
	role="none"
	class="resizeable-hitbox"
></div>

<svelte:body
	on:mousemove={(e) => {
		if (draggingWindow && e.clientX > 90 && e.clientX < 500) {
			sidebarWidth = `${e.clientX}px`;
		}
	}}
/>

<style>
	/* TODO: Resizeable sidebar, make file nav rendering more elegant */
	.side-bar {
		background-color: var(--background-1);
		width: var(--sidebar-width);
		height: 100vh;
		color: var(--foreground-0);
		font-family: var(--font-family);
		overflow-y: scroll;
	}

	.resizeable-hitbox {
		left: calc(var(--sidebar-width) - 0.25rem);
		position: absolute;
		height: 100vh;
		width: 0.5rem;
		cursor: col-resize;
	}

	.directory-nav {
		margin-top: 2rem;
		overflow-x: hidden;
		/* overflow-y: scroll; */
	}
</style>
