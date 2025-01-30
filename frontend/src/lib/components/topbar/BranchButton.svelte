<!-- BranchButton.svelte -->
<script lang="ts">
	import {
		branchName,
		documentTree,
		currentFile,
		editorText,
		apiAddress,
		allBranches,
		baseBranch
	} from '$lib/main';
	import { onMount } from 'svelte';
	import { ToastType, addToast } from '$lib/toast';
	import { cache } from '$lib/cache';
	import type { Branch } from '$lib/types';
	import LoadingIcon from '../elements/LoadingIcon.svelte';

	let showMenu = $state(false);
	let newBranchName: string = $state('');
	let showInput = $state(false);
	let showLoadingIcon: boolean = $state(false);

	/**
	 * Fetches the list of branches from the GitHub API, categorizing them as protected or non-protected.
	 *
	 * This asynchronous function sends a GET request to the specified GitHub repository's API endpoint
	 * to retrieve the branches. The function handles both successful and failed requests:
	 * - On success, it returns a promise that resolves to an object containing two arrays:
	 *   - `nonProtectedBranches`: A list of branches that are not protected.
	 *   - `protectedBranches`: A list of branches that are marked as protected.
	 * - On failure, it returns an object with empty arrays for both `nonProtectedBranches` and `protectedBranches`.
	 *
	 * @returns {Promise<{ nonProtectedBranches: Branch[]; protectedBranches: Branch[]; }>} A promise that resolves to an object with two properties:
	 * - `nonProtectedBranches`: An array of non-protected branches.
	 * - `protectedBranches`: An array of protected branches.
	 * If the request fails, both arrays will be empty.
	 *
	 * @throws {Error} Will throw an error if the response from the API is unsuccessful, with details including the status code and error message from the server.
	 *
	 * @example
	 * async function main() {
	 *   try {
	 *     const { nonProtectedBranches, protectedBranches } = await fetchExistingBranches();
	 *   } catch (err) {
	 *     console.error('Error fetching branches:', err);
	 *   }
	 * }
	 */
	async function fetchExistingBranches() {
		const response = await fetch(`${apiAddress}/api/branches`, {
			method: 'GET',
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json'
			}
		});

		// Check if the response is successful
		if (!response.ok) {
			const errorMessage = await response.json();
			console.error('Failed to fetch branches:', errorMessage);
			addToast(
				`Error fetching branches: ${response.statusText}. ${JSON.stringify(errorMessage)}`,
				ToastType.Error
			);
			return [];
		}

		const data = await response.json();
		const formattedBranches: Branch[] = data.branches.map((branch: string) => ({
			name: branch.replace(' (protected)', ''),
			isProtected: branch.includes('(protected)')
		}));
		allBranches.set(formattedBranches);
	}

	async function fetchDefaultBranch() {
		const response = await fetch(`${apiAddress}/api/repos/default-branch`);

		if (response.ok) {
			const data = await response.json();
			// Set the default branch to the baseBranch store
			baseBranch.set(data);
		} else {
			console.error('Failed to fetch default branch:', response.statusText);
		}
	}

	async function fetchCurrentBranch() {
		try {
			const response = await fetch(`${apiAddress}/api/current-branch`);

			if (response.ok) {
				const data = await response.json();
				branchName.set(data);
			} else {
				console.error('Failed to fetch current branch:', response.statusText);
			}
		} catch (error) {
			console.error('Error fetching current branch:', error);
		}
	}

	onMount(async () => {
		fetchDefaultBranch();
		fetchCurrentBranch();
		await fetchExistingBranches();
	});

	async function setBranchName(input: string) {
		if (!input) return;

		// Define validation rules
		const maxLength = 255; // Maximum length for branch name
		const invalidCharacters = /[~^:?*<>|]/; // Invalid special characters
		const startsWithLetterOrNumber = /^[a-zA-Z0-9]/; // Starts with letter or number
		const containsSpaces = /\s/; // Contains spaces

		// Define an array of rules
		const rules = [
			'Branch names must start with a letter or a number.',
			'Branch names cannot contain spaces. Use dashes (-) or underscores (_) instead.',
			'Branch names cannot contain special characters like ~, ^, :, ?, *, and others.',
			`Branch names must be shorter than ${maxLength} characters.`
		];

		const isValidBranchName = (name: string) => {
			return (
				!containsSpaces.test(name) &&
				startsWithLetterOrNumber.test(name) &&
				!invalidCharacters.test(name) &&
				name.length <= maxLength
			);
		};

		// Validate branch name
		if (!isValidBranchName(input)) {
			addToast(
				'Please ensure your branch name follows these rules:\n' + rules.join('\n'),
				ToastType.Warning
			);
			return;
		}

		if ($allBranches.some((branch) => branch.name === input && branch.isProtected)) {
			addToast(
				'Please select an existing branch name from the list of non-protected branches.\nYou can also create your own.',
				ToastType.Warning,
				true,
				1800
			);
			return;
		}

		showLoadingIcon = true;

		// Set branch name and reset state
		branchName.set(input);
		newBranchName = '';
		showMenu = false;

		if (!$allBranches.some((branch) => branch.name === input)) {
			addToast(`Now working on a new branch: "${input}".`, ToastType.Success, true, 1800);
			showLoadingIcon = false;
			return;
		}

		// Call backend to update working directory by checking out the branch
		const response = await fetch(`${apiAddress}/api/checkout/branches/${input}`, {
			method: 'PUT',
			credentials: 'include'
		});

		if (!response.ok) {
			addToast(
				`Failed to check out branch. Error ${response.status}: ${response.statusText}`,
				ToastType.Error
			);
			showLoadingIcon = false;
			return;
		}

		// After checking out, call the pull endpoint
		const pullResponse = await fetch(`${apiAddress}/api/pull/${input}`, {
			method: 'POST',
			credentials: 'include'
		});

		if (pullResponse.ok) {
			addToast(
				`Branch "${input}" checked out and updated successfully.`,
				ToastType.Success,
				true,
				1200
			);
		} else {
			addToast(`Failed to pull latest changes for branch "${input}".`, ToastType.Error);
			showLoadingIcon = false;
			return;
		}
		// Fetch the updated document tree after pulling changes
		const treeResponse = await fetch(`${apiAddress}/api/tree/doc`, {
			method: 'GET',
			credentials: 'include'
		});

		if (treeResponse.ok) {
			const updatedTree = await treeResponse.json();
			documentTree.set(updatedTree); // Update the store with the new tree

			cache.flush();

			// After updating the tree, check if there's a current file
			const currentFilePath = $currentFile;
			if (currentFilePath) {
				// Fetch the content of the current file
				const fileContentResponse = await fetch(
					`${apiAddress}/api/doc?path=${encodeURIComponent(currentFilePath)}`,
					{
						method: 'GET',
						credentials: 'include'
					}
				);

				if (fileContentResponse.ok) {
					const fileContent = await fileContentResponse.json(); // Get the content of the file
					editorText.set(fileContent.contents); // Update the editor text
					cache.set(currentFilePath, fileContent.contents);
				} else {
					console.error(
						'Failed to fetch the file content:',
						fileContentResponse.status,
						fileContentResponse.statusText
					);
				}
			}
		} else {
			console.error(
				'Failed to fetch updated document tree:',
				treeResponse.status,
				treeResponse.statusText
			);
		}
		showLoadingIcon = false;
	}

	function toggleMenu() {
		showMenu = !showMenu;
	}

	function closeMenu() {
		showMenu = false;
		showInput = false;
	}
</script>

<div class="branch-dropdown">
	<button onclick={toggleMenu} class="branch-button">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			fill="currentColor"
			width="18px"
			height="18px"
			viewBox="0 0 512 512"
		>
			<path
				d="M416,160a64,64,0,1,0-96.27,55.24c-2.29,29.08-20.08,37-75,48.42-17.76,3.68-35.93,7.45-52.71,13.93V151.39a64,64,0,1,0-64,0V360.61a64,64,0,1,0,64.42.24c2.39-18,16-24.33,65.26-34.52,27.43-5.67,55.78-11.54,79.78-26.95,29-18.58,44.53-46.78,46.36-83.89A64,64,0,0,0,416,160ZM160,64a32,32,0,1,1-32,32A32,32,0,0,1,160,64Zm0,384a32,32,0,1,1,32-32A32,32,0,0,1,160,448ZM352,192a32,32,0,1,1,32-32A32,32,0,0,1,352,192Z"
			/>
		</svg>
		{$branchName.length > 100 ? `${$branchName.slice(0, 100)}...` : $branchName}
	</button>

	{#if showMenu}
		<div class="branch-menu">
			<div class="branch-menu-header">
				<h4>Select or Create a Branch</h4>
				<button class="close-button" onclick={closeMenu} aria-label="Close menu">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						height="1.5rem"
						viewBox="0 -960 960 960"
						width="1.5rem"
						role="none"
					>
						<title>Exit</title>
						<path
							d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"
						/>
					</svg>
				</button>
			</div>
			<ul class="branch-list">
				{#each $allBranches.sort((a, b) => a.name.localeCompare(b.name)) as branch}
					<li>
						<button
							class="branch-option"
							onclick={() => setBranchName(branch.name)}
							onkeydown={(e) => e.key === 'Enter' && setBranchName(branch.name)}
							aria-label={`Select branch ${branch.name}`}
							disabled={branch.isProtected}
							class:protected={branch.isProtected}
						>
							{branch.name}
						</button>
					</li>
				{/each}
				{#if $allBranches.length === 0}
					<li>No branches available</li>
				{/if}

				<!-- "+" button to create a new branch -->
				<li>
					{#if !showInput}
						<button
							class="branch-option"
							onclick={() => {
								showInput = true;
								newBranchName = '';
							}}
							aria-label="Create new branch"
						>
							+ Create New Branch
						</button>
					{:else}
						<input
							type="text"
							bind:value={newBranchName}
							onkeydown={(e) => {
								if (e.key === 'Enter') {
									setBranchName(newBranchName); // Call the function to set the branch name
									newBranchName = ''; // Reset input field after setting the branch name
									showInput = false; // Hide input after creating
								}
							}}
							placeholder="Enter new branch name"
							class="new-branch-input"
						/>
					{/if}
				</li>
			</ul>
		</div>
	{/if}
</div>
<LoadingIcon bind:visible={showLoadingIcon} />

<style>
	.branch-dropdown {
		position: relative;
	}

	.branch-button {
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: transparent;
		color: var(--foreground-3);
		border-radius: 0.3rem;
		padding: 0.5rem 1rem;
		cursor: pointer;
		margin: 0 auto;
		margin-right: 1rem;
		margin-left: 1rem;
		font-size: 1.25rem;
	}

	.branch-button svg {
		margin-right: 0.25rem;
		width: 1.5rem;
		height: 1.5rem;
	}

	.branch-button:hover {
		background-color: var(--background-1);
		transition: background-color 0.3s ease;
	}

	.branch-menu {
		position: absolute;
		background-color: var(--background-1);
		border: 1px solid var(--foreground-3);
		padding: 1rem;
		z-index: 1000;
		min-width: 225px;
	}

	.branch-menu-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.branch-menu h4 {
		color: var(--foreground-0);
		margin: 0;
		font-size: 1rem;
	}

	.branch-list {
		max-height: 200px;
		overflow-y: scroll;
		margin: 0;
		padding: 0 0 0.25rem;
		list-style: none;
	}

	.close-button {
		background: transparent;
		border: none;
		cursor: pointer;
		position: absolute;
		top: 0.25rem;
		right: 0.25rem;
		padding: 0.25rem;
	}

	.close-button path {
		fill: var(--red);
	}

	.branch-option {
		padding: 0.5rem 0.5rem 0.5rem 1rem;
		cursor: pointer;
		width: 100%;
		text-align: left;
		box-sizing: border-box;
		color: var(--foreground-0);
	}

	.branch-option:hover {
		background-color: var(--background-0);
	}

	.branch-option.protected {
		color: var(--foreground-3);
		cursor: not-allowed;
	}

	input {
		padding: 0.5rem 0.1rem;
		padding-left: 0.5rem;
		border: 0.1rem solid var(--foreground-3);
		border-radius: 0.3rem;
		width: calc(100% - 1.5rem);
		box-sizing: border-box;
		background-color: var(--background-0);
		color: var(--foreground-0);
		margin: 0.5rem;
	}

	button {
		margin-top: 0.5rem;
		padding: 0.5rem 1rem;
		background-color: var(--background-1);
		border: none;
		border-radius: 0.3rem;
		color: var(--foreground-0);
		cursor: pointer;
	}

	button:hover {
		background-color: var(--background-2);
	}
</style>
