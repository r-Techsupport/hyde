<script lang="ts">
	import SideBar from '$lib/components/sidebar/SideBar.svelte';
	import FileNavigation from '$lib/components/sidebar/FileNavigation.svelte';
	import TopBar from '$lib/components/topbar/TopBar.svelte';
	import ChangeDialogue from '../lib/components/elements/ChangeDialogue.svelte';
	import { renderMarkdown } from '$lib/render';
	import { cache } from '$lib/cache';
	import LoadingIcon from '../lib/components/elements/LoadingIcon.svelte';
	import { ToastType, addToast } from '$lib/toast';
	import Toasts from '../lib/components/elements/Toasts.svelte';
	import {
		currentFile,
		me,
		branchName,
		documentTree,
		editorText,
		apiAddress,
		assetTree,
		allBranches
	} from '$lib/main';
	import { onMount } from 'svelte';
	import { dev } from '$app/environment';
	import SettingsMenu from '$lib/components/topbar/SettingsMenu.svelte';
	import AdminDashboard from '$lib/components/dashboard/AdminDashboard.svelte';
	import { Permission } from '$lib/types';
	import DocumentEditor from '$lib/components/editors/DocumentEditor.svelte';
	import AssetSelector from '$lib/components/sidebar/AssetSelector.svelte';
	import MockDirectory from '$lib/components/sidebar/MockDirectory.svelte';
	import { SelectedMode } from '$lib/main';
	import AssetEditor from '$lib/components/editors/AssetEditor.svelte';

	let mode = $state(SelectedMode.Documents);
	// TODO: figure out how to move this out of +page.svelte and into the document editor
	/** A reference to the div where markdown is rendered to */
	let previewWindow: HTMLElement | undefined = $state();

	onMount(async () => {
		const response = await fetch(`${apiAddress}/api/tree/doc`);
		const fetchedRootNode = await response.json();
		documentTree.set(fetchedRootNode); // Update the store with the fetched data
	});

	let showChangeDialogue: boolean = $state(false);
	let showLoadingIcon: boolean = $state(false);
	let showSettingsMenu: boolean = $state(false);
	let adminDashboardDialog: HTMLDialogElement | undefined = $state();
	let showEditor: boolean = $state(false);
	/** The path to the currently selected assets folder */
	let assetFolderPath = $state('');

	async function documentSelectionHandler(path: string) {
		// If the file in cache doesn't differ from the editor or no file is selected, there are no unsaved changes
		if ($currentFile === '' || (await cache.get($currentFile)) === $editorText) {
			showEditor = true;
			currentFile.set(path);
			editorText.set(
				(await cache.get(path)) ??
					'Something went wrong, the file tree reported by the backend references a nonexistent file.'
			);
			// non-null assertion: The above code shows the preview window
			renderMarkdown($editorText, previewWindow!);
		} else if (path === $currentFile) {
			// Do nothing
		} else {
			// Unsaved changes
			showChangeDialogue = true;
		}
	}

	let saveChangesHandler = $state(async (commitMessage: string): Promise<void> => {
		showLoadingIcon = true;

		const branch = $allBranches.find((b) => b.name === $branchName);

		if (branch && branch.isProtected) {
			addToast(
				`The branch '${$branchName}' is protected and cannot be modified.`,
				ToastType.Warning,
				true
			);
			showLoadingIcon = false; // Ensure loading icon is hidden
			return;
		}

		const response = await fetch(`${apiAddress}/api/doc`, {
			method: 'PUT',
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				contents: $editorText,
				path: $currentFile,
				commit_message: commitMessage,
				branch_name: $branchName
			})
		});
		showLoadingIcon = false;
		cache.flush();
		switch (response.status) {
			case 201:
				addToast('Changes synced successfully.', ToastType.Success);
				break;
			default:
				addToast(
					`An error was encountered syncing changes, please report to the developer (Code ${response.status}: "${response.statusText}").`,
					ToastType.Error,
					true
				);
		}
	});

	interface Props {
		/** The width of the sidebar */
		sidebarWidth?: string;
	}

	let { sidebarWidth = $bindable('14rem') }: Props = $props();

	onMount(async () => {
		// Fetch the document tree
		const docResponse = await fetch(`${apiAddress}/api/tree/doc`);
		documentTree.set(await docResponse.json());

		// Fetch the asset tree
		const assetResponse = await fetch(`${apiAddress}/api/tree/asset`);
		assetTree.set(await assetResponse.json());
	});

	onMount(async () => {
		// Check to see if the username cookie exists, it's got the same expiration time as the auth token but is visible to the frontend
		if (!document.cookie.includes('username')) {
			addToast(
				'You need to be logged in to access this page, redirecting...',
				ToastType.Error,
				false
			);
			setTimeout(() => {
				// TODO: When .html stripping middleware is complete, change this to always redirect to /login`
				if (dev) {
					window.location.href = '/login';
				} else {
					window.location.href = '/login.html';
				}
			}, 800);
			return;
		}
		const loginResponse = await fetch(`${apiAddress}/api/users/me`, { credentials: 'include' });
		// Unauthorized, need to login
		if (loginResponse.status === 401) {
			addToast('Your login has expired, redirecting...', ToastType.Error, false);
			setTimeout(() => {
				// TODO: When .html stripping middleware is complete, change this to always redirect to /login`
				if (dev) {
					window.location.href = '/login';
				} else {
					window.location.href = '/login.html';
				}
			}, 800);
			return;
		}
		me.set(await (await fetch(`${apiAddress}/api/users/me`, { credentials: 'include' })).json());
		me.subscribe((me) => {
			if (me.id === -1) {
				return;
			}
			if (me.permissions.includes(Permission.ManageContent)) {
				showEditor = true;
			}
		});
	});
</script>

<div style="--sidebar-width: {sidebarWidth}" class="container">
	<Toasts />
	<SideBar bind:sidebarWidth>
		<div class="directory-nav">
			<!-- TODO: migrate this stuff away from page.svelte, probably into the sidebar-->
			{#if mode === SelectedMode.Documents}
				<FileNavigation fileSelectHandler={documentSelectionHandler} {...$documentTree} />
			{:else}
				<!-- Display a button that switches the mode to docs -->
				<MockDirectory
					onclick={() => {
						mode = SelectedMode.Documents;
					}}
					label="docs"
				/>
			{/if}
			{#if mode === SelectedMode.Assets}
				<AssetSelector bind:mode bind:assetFolderPath />
			{:else}
				<MockDirectory
					onclick={() => {
						mode = SelectedMode.Assets;
					}}
					label="assets"
				/>
			{/if}
		</div>
	</SideBar>
	<div style="display: flex; flex-direction: column; height: 100vh; width: 100%;">
		<TopBar
			on:settingsopen={() => {
				showSettingsMenu = true;
			}}
		/>
		<SettingsMenu
			bind:visible={showSettingsMenu}
			on:showadmindashboard={() => {
				adminDashboardDialog!.showModal();
			}}
		/>
		{#if mode === SelectedMode.Documents}
			{#if showEditor && $currentFile !== ''}
				<DocumentEditor bind:saveChangesHandler bind:previewWindow={previewWindow!} />
			{:else}
				<span class="nofile-placeholder">
					<p>
						No file selected, please select a file to start editing. If you're unable to select a
						file, you might be missing the required permissions.
					</p>
				</span>
			{/if}
		{:else if mode === SelectedMode.Assets}
			<AssetEditor bind:assetFolderPath />
		{/if}
	</div>
	<LoadingIcon bind:visible={showLoadingIcon} />
	<ChangeDialogue bind:visible={showChangeDialogue} />
	<AdminDashboard bind:dialog={adminDashboardDialog!} />
</div>

<style>
	.container {
		background-color: var(--background-0);
		display: flex;
	}

	.nofile-placeholder {
		color: var(--foreground-3);
		display: flex;
	}

	.nofile-placeholder p {
		margin: 10%;
		margin-top: 5%;
	}
</style>
