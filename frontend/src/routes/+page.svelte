<script lang="ts">
	import SideBar from '$lib/components/SideBar.svelte';
	import FileNavigation from '$lib/components/FileNavigation.svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import ChangeDialogue from './ChangeDialogue.svelte';
	import { renderMarkdown } from '$lib/render';
	import { cache } from '$lib/cache';
	import { apiAddress } from '$lib/net';
	import LoadingIcon from './LoadingIcon.svelte';
	import { ToastType, addToast } from '$lib/toast';
	import Toasts from './Toasts.svelte';
	import { currentFile, me } from '$lib/main';
	import { get } from 'svelte/store';
	import { onMount } from 'svelte';
	import { dev } from '$app/environment';
	import SettingsMenu from '$lib/components/SettingsMenu.svelte';
	import AdminDashboard from '$lib/components/dashboard/AdminDashboard.svelte';
	import Editor from '$lib/components/Editor.svelte';
	import { Permission } from '$lib/types.d';

	/** The text currently displayed in the editing window */
	let editorText = '';
	/** A reference to the div where markdown is rendered to */
	let previewWindow: HTMLElement;
	/** The width of the sidebar */
	export let sidebarWidth = '14rem';
	/**
	 * The time in milliseconds that must pass after a keypress
	 * before markdown is rendered
	 */
	const DEBOUNCE_TIME: number = 500;
	let lastKeyPressedTime = Date.now();

	/** The base directory for filesystem navigation */
	let rootNode = {
		name: '',
		children: []
	};
	onMount(async () => {
		const response = await fetch(`${apiAddress}/api/tree`);
		rootNode = await response.json();
	});

	/**
	 * This function is called whenever a key is pressed.
	 *
	 * @see https://svelte.dev/repl/162005fa12cc4feb9f668e09260595a7?version=3.24.1
	 */
	async function onKeyDown() {
		lastKeyPressedTime = Date.now();
		setTimeout(() => {
			if (lastKeyPressedTime + DEBOUNCE_TIME >= Date.now()) {
				renderMarkdown(editorText, previewWindow);
			}
		}, DEBOUNCE_TIME);
	}

	let showChangeDialogue: boolean;
	let showLoadingIcon: boolean;
	let showSettingsMenu: boolean;
	let adminDashboardDialog: HTMLDialogElement;
	let showEditor: boolean = false;

	async function fileSelectionHandler(e: CustomEvent) {
		// If the file in cache doesn't differ from the editor or no file is selected, there are no unsaved changes
		if (get(currentFile) === '' || (await cache.get(get(currentFile))) === editorText) {
			showEditor = true;
			currentFile.set(e.detail.path);
			editorText =
				(await cache.get(e.detail.path)) ??
				'Something went wrong, the file tree reported by the backend references a nonexistent file.';
			renderMarkdown(editorText, previewWindow);
		} else if (e.detail.path === get(currentFile)) {
			// Do nothing
		} else {
			// Unsaved changes
			showChangeDialogue = true;
		}
	}

	let saveChangesHandler = async (commitMessage: string): Promise<void> => {
		showLoadingIcon = true;
		let response = await fetch(`${apiAddress}/api/doc`, {
			method: 'PUT',
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				contents: editorText,
				path: get(currentFile),
				commit_message: commitMessage
			})
		});
		showLoadingIcon = false;
		cache.flush();
		switch (response.status) {
			case 201:
				addToast({
					message: 'Changes synced successfully.',
					type: ToastType.Success,
					dismissible: true,
					timeout: 3000
				});
				break;
			default:
				addToast({
					message: `An error was encountered syncing changes, please report to the developer (Code ${response.status}: "${response.statusText}").`,
					type: ToastType.Error,
					dismissible: true
				});
		}
	};

	onMount(async () => {
		// Check to see if the username cookie exists, it's got the same expiration time as the auth token but is visible to the frontend
		if (!document.cookie.includes('username')) {
			addToast({
				message: 'You need to be logged in to access this page, redirecting...',
				type: ToastType.Error,
				dismissible: false
			});
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
			addToast({
				message: 'Your login has expired, redirecting...',
				type: ToastType.Error,
				dismissible: false
			});
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
			<FileNavigation on:fileselect={fileSelectionHandler} {...rootNode} />
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
				adminDashboardDialog.showModal();
			}}
		/>
		{#if showEditor && $currentFile !== ''}
			<Editor bind:saveChangesHandler bind:editorText bind:previewWindow />
		{:else}
			<span class="nofile-placeholder">
				<p>
					No file selected, please select a file to start editing. If you're unable to select a
					file, you might be missing the required permissions.
				</p>
			</span>
		{/if}
	</div>
	<LoadingIcon bind:visible={showLoadingIcon} />
	<ChangeDialogue bind:visible={showChangeDialogue} />
	<AdminDashboard bind:dialog={adminDashboardDialog} />
</div>

<svelte:window on:keydown={onKeyDown} />

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
