<script lang="ts">
	interface Props {
		/** Binding to the css variable determining sidebar width */
		sidebarWidth: string;
		children?: import('svelte').Snippet;
	}

	let { sidebarWidth = $bindable(), children }: Props = $props();
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
	let draggingWindow = $state(false);
</script>

<div class="side-bar">
	<img src="hyde-assets/logo-dark.svg" alt="Logo" />
	{@render children?.()}
</div>

<div
	onmousedown={() => {
		draggingWindow = true;
	}}
	onmouseup={() => {
		draggingWindow = false;
	}}
	role="none"
	class="resizeable-hitbox"
></div>

<svelte:body
	onmousemove={(e) => {
		if (draggingWindow && e.clientX > 90 && e.clientX < 500) {
			sidebarWidth = `${e.clientX}px`;
		}
	}}
/>

<style>
	/* TODO: Resizeable sidebar, make file nav rendering more elegant */
	.side-bar {
		flex-shrink: 0;
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
