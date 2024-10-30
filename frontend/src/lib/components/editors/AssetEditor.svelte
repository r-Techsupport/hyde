<script lang="ts">
	import type { INode } from '$lib/main';
	import { apiAddress, assetTree } from '$lib/main';
	import { onMount, tick } from 'svelte';
	import { blur } from 'svelte/transition';

	export let assetFolderPath = '';
	let uploadedFiles: FileList;

	$: {
		uploadedFiles;
		fileUploadHandler();
	}


	async function fileUploadHandler() {
		if (uploadedFiles && uploadedFiles.length > 0) {
			const file = uploadedFiles.item(0)!;
			console.log(file);
			// TODO: increase the max body limit or
			// switch to multipart forms or something
			fetch(`${apiAddress}/api/asset/${assetFolderPath}/${file.name}`, {
				method: 'PUT',
				credentials: 'include',
				headers: { 'Content-Type': 'application/octet-stream' },
				body: await file.arrayBuffer()
			});
		}
	}

	/**
	 * When an image is being displayed in "full screen mode", this is the path of that
	 * image. Otherwise, it's an empty string.
	 */
	let fullScreenImagePath = '';
	let fullScreenImage: HTMLImageElement;
	onMount(() => {

		// fullScreenImage.loading = 'eager'  URL.revokeObjectURL(url);
		// console.log(fullScreenImage.naturalWidth + 'x' + fullScreenImage.naturalHeight);
		// console.log(fullScreenImage);
	});
	let fullScreenHttpInfo: Response | undefined;
	// TODO: there's a race condition here that makes image
	// resolution only load *sometimes*
	async function loadHttpInfo() {
		while (!fullScreenImage.complete) {
			await tick();
		}
		setTimeout(async () => {
			// if (fullScreenImage.complete) {
				fullScreenHttpInfo = await fetch(`${apiAddress}/api/asset/${fullScreenImagePath}`, {method: "GET", headers: {
			"Accept": "image/*"
				}});
			// }
		}, 100);
	}
	$: {
		if (fullScreenImagePath !== '') {
			fetch(`${apiAddress}/api/asset/${fullScreenImagePath}`).then(async (r) => {
				fullScreenHttpInfo = r;
				const objectUrl = URL.createObjectURL(await r.blob());
				fullScreenImage.onload = () => {
					URL.revokeObjectURL(objectUrl);
				};
				fullScreenImage.src = objectUrl;
			});

			loadHttpInfo();
		}
	}

	let tree: INode = {
		name: 'loading',
		children: []
	};

	assetTree.subscribe((t) => (tree = t));
</script>

{#if fullScreenImagePath !== ''}
	<div
		class="fullscreen-backdrop"
		transition:blur={{ duration: 100 }}
		on:click={() => {
			fullScreenImagePath = '';
		}}
		on:keydown={() => {
			fullScreenImagePath = '';
		}}
		role="none"
	></div>
	<div class="fullscreen-preview" transition:blur={{ duration: 100 }}>
		<div class="fullscreen-content">
			<img
				bind:this={fullScreenImage}
				class="fullscreen-img"
				src={`${apiAddress}/api/asset/${fullScreenImagePath}`}
				loading="eager"
				alt={`${fullScreenImagePath}`}
			/>
			<div class="fullscreen-info">
				<h2>{fullScreenImagePath.split('/')[1]}</h2>
				<p>
					<strong>Resolution:</strong>
					<code>{fullScreenImage.naturalWidth}x{fullScreenImage.naturalHeight}</code>
				</p>
				<!-- {#await fullScreenHttpInfo}
					<p>Loading more info...</p>
				{:then httpInfo} -->
				{#if fullScreenHttpInfo}
					<p>
						<strong>Encoding:</strong> <code>{fullScreenHttpInfo.headers.get('Content-Type')}</code>
					</p>
					<p>
						<strong>File size:</strong>
						<code
							>{(Number(fullScreenHttpInfo.headers.get('Content-Length')) / 1000).toLocaleString(
								'EN-us',
								{
									useGrouping: 'always'
								}
							)}kB</code
						>
					</p>
				{/if}
				<!-- {/await} -->
			</div>
		</div>
	</div>
{/if}

<div class="status-bar">
	<!-- TODO: better message for no directory selected -->
	<p>You're viewing the assets for <strong>"{assetFolderPath}"</strong></p>
</div>

<div class="asset-catalogue">
	{#each tree.children.find((n) => n.name === assetFolderPath)?.children ?? [] as asset}
		<button
			on:click={() => {
				fullScreenImagePath = `${assetFolderPath}/${asset.name}`;
			}}
			class="asset"
			title={asset.name}
		>
			<img
				src={`${apiAddress}/api/asset/${assetFolderPath}/${asset.name}`}
				alt={`${assetFolderPath}/${asset.name}`}
			/>
			<code>{asset.name}</code>
		</button>
	{/each}
	<input bind:files={uploadedFiles} type="file" id="upload-new" style="display: none" />
	<label for="upload-new" class="asset upload-new" title="Upload new asset">
		<svg xmlns="http://www.w3.org/2000/svg" height="80%" viewBox="0 -960 960 960" width="80%"
			><path d="M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z" /></svg
		>
		<code>Upload new asset</code>
	</label>
</div>

<style>
	:root {
		font-family: var(--font-family);
		color: var(--foreground-2);
	}

	.asset,
	.asset::before,
	.asset::after,
	.upload-new,
	.upload-new::before,
	.upload-new::after,
	.asset-catalogue,
	.asset-catalogue::before,
	.asset-catalogue::after {
		box-sizing: border-box;
	}

	.asset-catalogue {
		display: inline-block;
		box-sizing: border-box;
		text-align: center;
		overflow: hidden scroll;
		margin: 0.3rem;
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
		border-radius: 5px;
		margin: 0.1rem;
		width: calc((100vw - var(--sidebar-width)) / 7 - 0.5rem);
		min-width: 200px;
		min-height: 200px;
		height: calc((100vw - var(--sidebar-width) - 0.5rem) / 7);
		transition: 0.3s;
		display: inline-block;
		vertical-align: top;
		text-align: center;
		text-overflow: ellipsis;
		overflow-x: hidden;
		border: 1px var(--background-3) solid;
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
		width: calc((100vw - var(--sidebar-width)) / 7 - 0.5rem);
		min-width: 200px;
		min-height: 200px;
		height: calc((100vw - var(--sidebar-width) - 0.5rem) / 7);
		fill: var(--background-3);
	}

	.upload-new code {
		font-family: var(--font-family);
		font-size: 1rem;
		position: relative;
		bottom: 1.5rem;
		color: var(--foreground-4);
		border-top: none;
	}

	.upload-new svg {
		object-fit: contain;
		height: calc(100% - 2.1rem);

		/* height: calc(100% -2rem); */
	}

	.status-bar {
		border: none;
		border-bottom: 0.05rem solid var(--foreground-4);
	}

	.status-bar p {
		padding-left: 0.3rem;
	}

	.fullscreen-backdrop {
		transition: opacity 5s ease-in-out;
		position: absolute;
		cursor: pointer;
		z-index: 1;
		left: 0;
		top: 0;
		width: 100vw;
		height: 100vh;
		background-color: rgb(0 0 0 / 61%);
		backdrop-filter: blur(5px);
	}

	.fullscreen-preview {
		position: absolute;
		display: flex;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		justify-content: center;
		align-items: center;
		z-index: 3;
		pointer-events: none;
	}

	.fullscreen-content {
		display: flex;
		flex-direction: row;
	}

	.fullscreen-img {
		border-radius: 5px;
		pointer-events: all;
		margin: 2rem;

		/* height: 80vh; */
		width: 40vw;
		max-height: 80vh;
		object-fit: contain;
	}

	.fullscreen-info {
		color: var(--foreground-1);
		width: 40vw;

		/* height: 60vh; */
		box-sizing: content-box;
		padding: 2rem;
	}

	.fullscreen-info h2 {
		color: var(--foreground-0);
	}
</style>
