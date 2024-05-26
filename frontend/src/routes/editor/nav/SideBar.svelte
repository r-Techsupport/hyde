<script lang="ts">
	import { apiAddress } from '$lib/net';
	import FileNavigation from './FileNavigation.svelte';
	import DOMPurify from 'dompurify';
	import { onMount } from 'svelte';

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

	onMount(async () => {
		// TODO: Dynamically determine whether to refer to local dev url
		// or to relative route
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

	.directory-nav {
		margin-top: 2rem;
		overflow-x: hidden;
		/* overflow-y: scroll; */
	}
</style>
