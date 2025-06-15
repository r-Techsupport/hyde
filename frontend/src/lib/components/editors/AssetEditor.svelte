<script lang="ts">
	import { apiAddress } from '$lib/main';
	import { assetTree } from '$lib/state/sidebar.svelte';
	import { addToast, ToastType } from '$lib/toast';
	import { blur } from 'svelte/transition';
	import ConfirmationDialogue from '../elements/ConfirmationDialogue.svelte';
	import LoadingIcon from '../elements/LoadingIcon.svelte';

	interface Props {
		assetFolderPath?: string;
	}

	let { assetFolderPath = $bindable('') }: Props = $props();
	let uploadedFiles: FileList | undefined = $state();

	async function fileUploadHandler() {
		if (uploadedFiles && uploadedFiles.length > 0) {
			loadingIconVisible = true;
			const file = uploadedFiles.item(0)!;
			const r = await fetch(`${apiAddress}/api/asset/${assetFolderPath}/${file.name}`, {
				method: 'PUT',
				credentials: 'include',
				headers: { 'Content-Type': 'application/octet-stream' },
				body: await file.arrayBuffer()
			});
			const reportedTree = await (await fetch(`${apiAddress}/api/tree/asset`)).json();
			assetTree.name = reportedTree.name;

			loadingIconVisible = false;
			if (r.ok) {
				addToast(`"${file.name}" was uploaded successfully`, ToastType.Info, true, 1500);
			} else {
				addToast(
					`Failed to upload file, please report issue to the developer`,
					ToastType.Error,
					true,
					1500
				);
			}
		}
	}

	/**
	 * When an image is being displayed in "full screen mode", this is the path of that
	 * image. Otherwise, it's an empty string.
	 */
	let fullScreenImagePath = $state('');
	let fullScreenImage: HTMLImageElement | undefined = $state();
	let width = $state(0);
	let height = $state(0);
	let fullScreenHttpInfo: Response | undefined = $state();
	// So basically, Svelte doesn't understand updates the browser makes to an image object,
	// so it doesn't react to changes. This is fixed by manually starting a polling cycle
	// that updates the image resolution when the image has finished loading
	function cb() {
		if (fullScreenImage?.complete) {
			width = fullScreenImage?.naturalWidth ?? 0;
			height = fullScreenImage?.naturalHeight ?? 0;
		} else {
			setTimeout(cb, 50);
		}
	}
	$effect(() => {
		// eslint-disable-next-line
		assetTree;
		fullScreenImagePath = '';
	});

	let deletionConfirmationVisible = $state(false);
	let loadingIconVisible = $state(false);
	// Whenever the list of uploaded files changes, call the handler to write new changes
	// to the git repo
	$effect(() => {
		// eslint-disable-next-line
		uploadedFiles;
		fileUploadHandler();
	});
	$effect(() => {
		if (fullScreenImagePath !== '') {
			fetch(`${apiAddress}/api/asset/${fullScreenImagePath}`).then(async (r) => {
				fullScreenHttpInfo = r;
				const objectUrl = URL.createObjectURL(await r.blob());
				// non-null assertion: Once the full screen image path is set, then a full screen image element is
				// defined
				fullScreenImage!.src = objectUrl;
			});
		}
		cb();
	});
</script>

{#if loadingIconVisible}
	<LoadingIcon bind:visible={loadingIconVisible} />
{/if}
<!-- Full screen image editor -->
{#if fullScreenImagePath !== ''}
	<div
		class="fullscreen-backdrop"
		transition:blur={{ duration: 100 }}
		onclick={() => {
			fullScreenImagePath = '';
		}}
		onkeydown={() => {
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
				alt={`${fullScreenImagePath}`}
			/>
			<div class="fullscreen-info">
				<h2>{fullScreenImagePath.split('/')[1]}</h2>
				<p>
					<strong>Resolution:</strong>
					<code>{width}x{height}</code>
				</p>
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
									useGrouping: 'always',
									maximumFractionDigits: 1
								}
							)}kB</code
						>
					</p>
				{/if}
				<button
					onclick={() => {
						deletionConfirmationVisible = true;
					}}
					class="delete-button"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						height="24px"
						viewBox="0 -960 960 960"
						width="24px"
						><path
							d="M280-120q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520ZM360-280h80v-360h-80v360Zm160 0h80v-360h-80v360ZM280-720v520-520Z"
						/></svg
					>
					Delete Image
				</button>
				{#if deletionConfirmationVisible}
					<ConfirmationDialogue
						bind:visible={deletionConfirmationVisible}
						confirmHandler={async () => {
							loadingIconVisible = true;
							const r = await fetch(`${apiAddress}/api/asset/${fullScreenImagePath}`, {
								method: 'DELETE',
								credentials: 'include'
							});
							if (r.ok) {
								addToast(
									`"${fullScreenImagePath}" was deleted successfully`,
									ToastType.Info,
									true,
									1500
								);
							} else {
								addToast(
									`Failed to delete file, please report issue to the developer`,
									ToastType.Error,
									true,
									1500
								);
							}
							const reportedTree = await (await fetch(`${apiAddress}/api/tree/asset`)).json();
							assetTree.name = reportedTree.name;
							assetTree.children = reportedTree.children;
							fullScreenImagePath = '';
							loadingIconVisible = false;
						}}
						><p>Are you sure you want to delete the file <code>{fullScreenImagePath}</code>?</p>
					</ConfirmationDialogue>
				{/if}
			</div>
		</div>
	</div>
{/if}

<!-- Catalogue and status bar, or placeholder for no assets -->
{#if assetFolderPath !== ''}
	<div class="status-bar">
		<p>You're viewing the assets for <strong>"{assetFolderPath}"</strong></p>
	</div>

	<div class="asset-catalogue">
		{#each assetTree.children.find((n) => n.name === assetFolderPath)?.children ?? [] as asset (asset.name)}
			<button
				onclick={() => {
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
{:else}
	<p class="noasset-placeholder">
		No folder selected, please select a folder to start managing assets.
	</p>
{/if}

<style>
	:root {
		font-family: var(--font-family);
		color: var(--foreground-2);
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
		width: 40vw;
		max-height: 80vh;
		object-fit: contain;
	}

	.fullscreen-info {
		color: var(--foreground-1);
		width: 40vw;
		pointer-events: all;
		box-sizing: content-box;
		padding: 2rem;
	}

	.fullscreen-info h2 {
		color: var(--foreground-0);
	}

	.delete-button {
		display: flex;
		align-items: center;
		cursor: pointer;
		fill: var(--red);
		color: var(--red);
		background-color: rgb(0 0 0 / 30%);
		backdrop-filter: blur(20px);
		border-radius: 5px;
		border: 1px solid var(--red);
		padding: 0.3rem 3rem;
		margin-left: -0.5rem;
	}

	.delete-button:hover {
		background-color: var(--red);
		fill: var(--background-0);
		color: var(--background-0);
	}

	.noasset-placeholder {
		margin: 10%;
		margin-top: 5%;
	}
</style>
