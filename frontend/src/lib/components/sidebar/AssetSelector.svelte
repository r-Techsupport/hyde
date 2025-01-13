<!-- The sidebar navigation for the asset editor -->
<script lang="ts">
	import { assetTree, SelectedMode, type INode } from '$lib/main';
	let { mode = $bindable(), assetFolderPath = $bindable() } = $props();

	/** The base directory for filesystem navigation */
	let tree: INode = $state({
		name: '',
		children: []
	});
	assetTree.subscribe((t) => (tree = t));

	function assetSelectionHandler(e: MouseEvent) {
		const selectedFolder = (e.currentTarget! as HTMLButtonElement).children[1].innerHTML;
		assetFolderPath = selectedFolder;
	}
</script>

<div class="container">
	<span
		onclick={() => {
			mode = SelectedMode.Documents;
		}}
		onkeydown={() => {
			mode = SelectedMode.Documents;
		}}
		class="label"
		role="button"
		tabindex="0"
	>
		<svg
			style="transform: rotate(90deg)"
			xmlns="http://www.w3.org/2000/svg"
			height="24px"
			viewBox="0 -960 960 960"
			width="24px"><path d="M504-480 320-664l56-56 240 240-240 240-56-56 184-184Z" /></svg
		>
		assets
	</span>
	<!-- TODO -->
	<!-- <input type="search" class="search-bar" placeholder="Filter folders..."/> -->
	{#each tree.children as node}
		{#if node.children.length > 0}
			<button onclick={assetSelectionHandler} class="directory-listing">
				<svg
					class="file-icon"
					xmlns="http://www.w3.org/2000/svg"
					height="24px"
					viewBox="0 -960 960 960"
					width="24px"
					><path
						d="M160-160q-33 0-56.5-23.5T80-240v-480q0-33 23.5-56.5T160-800h240l80 80h320q33 0 56.5 23.5T880-640v400q0 33-23.5 56.5T800-160H160Zm0-80h640v-400H447l-80-80H160v480Zm0 0v-480 480Z"
					/></svg
				>
				<p>{node.name}</p>
			</button>
		{/if}
	{/each}
</div>

<style>
	.container {
		display: flex;
		flex-direction: column;
		align-items: center;
		color: var(--foreground-0);
	}

	.label {
		box-sizing: border-box;
		padding-left: 1rem;
		display: flex;
		cursor: pointer;
		border: none;
		background: none;
		color: var(--foreground-0);
		font-size: inherit;

		/* Sizing, spacing */
		width: 98%;
		border-radius: 5px;
		margin-top: 0.4rem;
		margin-bottom: 0.4rem;
		padding-top: 0.4rem;
		padding-bottom: 0.4rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.directory-listing {
		background-color: transparent;
		border-style: none;
		font-size: inherit;
		color: inherit;
		display: flex;
		flex-direction: row;
		align-items: center;
		width: 98%;
		margin: 0.1rem;

		/* margin-top: 0.4rem;
		margin-bottom: 0.4rem; */
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		border-radius: 0.5rem;
	}

	.directory-listing:hover {
		background-color: var(--background-2);
		cursor: pointer;
	}

	.file-icon {
		fill: var(--foreground-4);
		margin-left: 1.25rem;
		margin-right: 0.5rem;
	}

	.label svg {
		fill: var(--foreground-5);
	}

	/* .search-bar {
		color: var(--foreground-0);
		background-color: var(--background-3);

		width: 95%;
		height: 2rem;
		border-style: none;
	}

	.search-bar::placeholder {
		padding-left: 0.5rem;
		color: var(--foreground-4);
	} */
</style>
