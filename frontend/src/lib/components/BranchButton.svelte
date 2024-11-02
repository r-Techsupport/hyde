<!-- BranchButton.svelte -->
<script lang="ts">
	import { branchName, documentTreeStore, currentFile, editorText } from '$lib/main';
	import { derived, get } from 'svelte/store';
	import { onMount } from 'svelte';
	import { apiAddress } from '$lib/net';
	import { ToastType, addToast } from '$lib/toast';
	import { cache } from '$lib/cache';

	let showMenu = false;
	let protectedBranches: Branch[] = [];
	let nonProtectedBranches: Branch[] = [];
	let newBranchName: string = '';
	let showInput = false;

	const currentBranch = derived(branchName, ($branchName) => $branchName);

	interface Branch {
		name: string;
		isProtected: boolean;
	}

	/**
	 * Fetches existing branches from the GitHub API.
	 *
	 * This asynchronous function sends a GET request to retrieve the list of branches
	 * from the specified GitHub repository. If the request is successful, it returns
	 * a promise that resolves to an array containing the branch data. If the request
	 * fails (e.g., due to network issues, authentication problems, or server errors),
	 * it returns a promise that rejects with an error object containing details about the failure.
	 *
	 * @returns {Promise<Array>} A promise that resolves to an array of branches on success,
	 * or rejects with an error object containing details about the failure.
	 *
	 * @throws Will throw an error if the response from the API is not successful. The error object
	 * will include the status code and any additional error message from the API.
	 *
	 * @example
	 * async function main() {
	 *   try {
	 *     const branches = await fetchExistingBranches();
	 *     console.log('Existing branches:', branches);
	 *   } catch (err) {
	 *     console.error('Failed to fetch branches:', err);
	 *   }
	 * }
	 */
	async function fetchExistingBranches(): Promise<{
		nonProtectedBranches: Branch[];
		protectedBranches: Branch[];
	}> {
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
			addToast({
				message: `Error fetching branches: ${response.statusText}. ${JSON.stringify(errorMessage)}`,
				type: ToastType.Error,
				dismissible: true
			});
			return { nonProtectedBranches: [], protectedBranches: [] };
		}

		// Extract and set the branches if the response is successful
		const data = await response.json();
		const branches: string[] = data.data?.branches || [];

		// Map through branches to create Branch objects
		const nonProtectedBranches: Branch[] = branches
			.filter((branch: string) => !branch.includes('(protected)'))
			.map((branch: string) => ({
				name: branch.split(' (')[0],
				isProtected: false
			}));

		const protectedBranches: Branch[] = branches
			.filter((branch: string) => branch.includes('(protected)'))
			.map((branch: string) => ({
				name: branch.split(' (')[0],
				isProtected: true
			}));

		return { nonProtectedBranches, protectedBranches };
	}

	onMount(async () => {
		const { nonProtectedBranches: fetchedNonProtected, protectedBranches: fetchedProtected } =
			await fetchExistingBranches();
		nonProtectedBranches = fetchedNonProtected;
		protectedBranches = fetchedProtected;
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
			addToast({
				message: 'Please ensure your branch name follows these rules:\n' + rules.join('\n'),
				type: ToastType.Warning,
				dismissible: true
			});
			return;
		}

		if (protectedBranches.some((branch) => branch.name === input)) {
			addToast({
				message:
					'Please select an existing branch name from the list of non-protected branches.\n You can also create your own',
				type: ToastType.Warning,
				dismissible: true,
				timeout: 1500
			});
			return;
		}

		// Set branch name and reset state
		branchName.set(input);
		newBranchName = '';
		showMenu = false;

		if (!nonProtectedBranches.some((branch) => branch.name === input)) {
			addToast({
				message: `Now working on a new branch: "${input}".`,
				type: ToastType.Success,
				dismissible: true,
				timeout: 1800
			});
			return;
		}

		// Call backend to update working directory by checking out the branch
		const response = await fetch(`${apiAddress}/api/checkout/branches/${input}`, {
			method: 'PUT',
			credentials: 'include'
		});

		if (!response.ok) {
			addToast({
				message: `Failed to check out branch. Error ${response.status}: ${response.statusText}`,
				type: ToastType.Error,
				dismissible: true
			});
			return;
		}

		// After checking out, call the pull endpoint
		const pullResponse = await fetch(`${apiAddress}/api/pull/${input}`, {
			method: 'POST',
			credentials: 'include'
		});

		if (!pullResponse.ok) {
			addToast({
				message: `Failed to pull latest changes for branch "${input}".`,
				type: ToastType.Error,
				dismissible: true
			});
			return;
		} else {
			addToast({
				message: `Branch "${input}" checked out and updated successfully.`,
				type: ToastType.Success,
				dismissible: true,
				timeout: 1200
			});
		}

		// Fetch the updated document tree after pulling changes
		const treeResponse = await fetch(`${apiAddress}/api/tree`, {
			method: 'GET',
			credentials: 'include'
		});

		if (treeResponse.ok) {
			const updatedTree = await treeResponse.json();
			documentTreeStore.set(updatedTree); // Update the store with the new tree

			cache.flush();

			// After updating the tree, check if there's a current file
			const currentFilePath = get(currentFile);
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
	<button on:click={toggleMenu} class="branch-button">
		{$currentBranch.length > 100 ? `${$currentBranch.slice(0, 100)}...` : $currentBranch}
	</button>

	{#if showMenu}
		<div class="branch-menu">
			<div class="branch-menu-header">
				<h4>Select Existing Branch</h4>
				<button class="close-button" on:click={closeMenu} aria-label="Close menu">
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
				{#each nonProtectedBranches as branch}
					<li>
						<button
							class="branch-option"
							on:click={() => setBranchName(branch.name)}
							on:keydown={(e) => e.key === 'Enter' && setBranchName(branch.name)}
							aria-label={`Select branch ${branch.name}`}
						>
							{branch.name}
						</button>
					</li>
				{/each}
				{#if nonProtectedBranches.length === 0}
					<li>No branches available</li>
				{/if}

				<!-- "+" button to create a new branch -->
				<li>
					{#if !showInput}
						<button
							class="branch-option"
							on:click={() => {
								showInput = true;
								newBranchName = '';
							}}
							aria-label="Create new branch"
						>
							+
						</button>
					{:else}
						<input
							type="text"
							bind:value={newBranchName}
							on:keydown={(e) => {
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

<style>
	.branch-dropdown {
		position: relative;
	}

	.branch-button {
		position: relative;
		background-color: var(--background-0);
		color: var(--foreground-0);
		border: none;
		border-radius: 0.3rem;
		padding: 0.5rem 1rem;
		cursor: pointer;
		font-size: medium;
		margin-right: 1rem;
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
		padding: 0;
		list-style: none;
	}

	.close-button {
		background: transparent;
		border: none;
		cursor: pointer;
		position: absolute;
		top: 0.01rem;
		right: 0.01rem;
	}

	.close-button path {
		fill: var(--red);
	}

	.branch-option {
		padding: 0.5rem 0.1rem;
		cursor: pointer;
		width: 100%;
		text-align: left;
		box-sizing: border-box;
		color: var(--foreground-0);
	}

	.branch-option:hover {
		background-color: var(--background-0);
	}

	input {
		padding: 0.5rem 0.1rem;
		border: 1px solid var(--foreground-3);
		border-radius: 0.3rem;
		width: calc(100% - 1.5rem);
		box-sizing: border-box;
		background-color: var(--background-0);
		color: var(--foreground-0);
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
