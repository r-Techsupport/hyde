<script lang="ts">
	import type { INode } from '$lib/main';
	import { apiAddress, assetTree } from '$lib/main';

	export let assetFolderPath = '';

    /**
     * When an image is being displayed in "full screen mode", this is the path of that
     * image. Otherwise, it's an empty string.
     */
    let fullScreenImagePath = '';
	let tree: INode = {
		name: 'loading',
		children: []
	};

	assetTree.subscribe((t) => (tree = t));
	if (assetFolderPath !== '') {
		let currentDir = tree.children.find((v) => v.name === assetFolderPath);
		console.log(currentDir?.children);
	}
</script>

<div class="status-bar">
	<!-- TODO: better message for no directory selected -->
	<p>You're viewing the assets for <strong>"{assetFolderPath}"</strong></p>
</div>

<div class="asset-catalogue">
	{#each tree.children.find((n) => n.name === assetFolderPath)?.children ?? [] as asset}
    <button class="asset" title="{asset.name}">
		<img
			src={`${apiAddress}/api/asset/${assetFolderPath}/${asset.name}`}
			alt={`${assetFolderPath}/${asset.name}`}
		/>
		<code>{asset.name}</code>
    </button>
	{/each}
    <div class="asset upload-new" title="Upload new asset">
        <svg xmlns="http://www.w3.org/2000/svg" height=80% viewBox="0 -960 960 960" width=80%><path d="M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z"/></svg>
        <p>Upload new asset</p>
    </div>
</div>

<style>
	:root {
		font-family: var(--font-family);
		color: var(--foreground-2);
	}

    .asset-catalogue {
        display: inline-block;
        box-sizing: border-box;
        text-align: center;
        overflow-x: hidden;
        overflow-y: scroll;
    }

    .asset-catalogue img {
        object-fit: contain;
        border-radius: 5px 5px 0 0;
        width: 100%;
        height: calc(100% - 2rem);
        border-bottom: 1px solid var(--foreground-5);
    }

    .asset {
        background-color: color-mix(in hsl, var(--background-0), var(--background-1));
        color: inherit;
        border: none;
        border-radius: 5px;
        margin: 0.1rem;
        width: calc((100vw - var(--sidebar-width)) / 7 - 0.5rem);
        min-width: 200px;
        min-height: 200px;
        height: calc((100vw - var(--sidebar-width) - 0.5rem) / 7);
        transition: 0.3s;
 
        display: inline-block;
        text-align: center;
        text-overflow: ellipsis;
        overflow-x: hidden;
    }

    .asset:hover {
        background-color: var(--background-1);
        cursor: pointer;
    }

    .asset code {
        width: 100%;
        max-height: 0.5rem;
        padding: 0;
        margin: 0.2rem;
        text-overflow: clip;
        white-space: nowrap;
    }

    .upload-new {
        fill: var(--background-3);
    }

    .upload-new p {
        margin-top: -0.1rem;
        color: var(--foreground-4)
    }

	.status-bar {
		border: none;
		border-bottom: 0.05rem solid var(--foreground-4);
	}

	.status-bar p {
		padding-left: 0.3rem;
	}
</style>
