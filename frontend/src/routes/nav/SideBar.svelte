<script lang="ts">
	/** Binding to the css variable determining sidebar width */
	export let sidebarWidth: string;
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
</script>

<div class="side-bar">
	<slot></slot>
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
</style>
