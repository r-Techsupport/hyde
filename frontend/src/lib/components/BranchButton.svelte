<!-- BranchButton.svelte -->
<script lang="ts">
	import { branchName, documentTreeStore, currentFile, editorText } from '$lib/main';
	import { derived, get } from 'svelte/store';
	import { onMount } from 'svelte';
	import { apiAddress } from '$lib/net';
	import { ToastType, addToast } from '$lib/toast';
	import { cache } from '$lib/cache';

	let showMenu = false;
	let protectedBranches: string[] = [];
	let nonProtectedBranches: string[] = [];
	let newBranchName: string = '';
	let showInput = false;

	const currentBranch = derived(branchName, ($branchName) => $branchName);

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
			// Handle error without try-catch
			response.json().then((errorMessage) => {
				console.error('Failed to fetch branches:', errorMessage);
				addToast({
					message: `Error fetching branches: ${response.statusText}. ${JSON.stringify(errorMessage)}`,
					type: ToastType.Error,
					dismissible: true
				});
			});
			return; // Exit if response is not OK
		}

		// Extract and set the branches if the response is successful
		response.json().then((data) => {
			if (data.data && data.data.branches) {
				// Map through branches to filter out protected ones and extract branch names
				nonProtectedBranches = data.data.branches
					.filter((branch: string) => !branch.includes('(protected)')) // Filter out protected branches
					.map((branch: string) => branch.split(' (')[0]); // Extract just branch names
				protectedBranches = data.data.branches
					.filter((branch: string) => branch.includes('(protected)')) // Filter out protected branches
					.map((branch: string) => branch.split(' (')[0]); // Extract just branch names
			} else {
				protectedBranches = []; // Reset if no branches found
				nonProtectedBranches = [];
			}
		});
	}

	onMount(() => {
		fetchExistingBranches();
	});

	async function setBranchName(input: string) {
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

		if (!input) return;

		// Validate branch name
		if (!isValidBranchName(input)) {
			addToast({
				message: 'Please ensure your branch name follows these rules:\n' + rules.join('\n'),
				type: ToastType.Warning,
				dismissible: true
			});
			return;
		}

		if (protectedBranches.includes(input)) {
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

		if (!nonProtectedBranches.includes(input)) {
			addToast({
				message: `Changed branch name to "${input}".`,
				type: ToastType.Success,
				dismissible: true,
				timeout: 1500
			});
			return;
		}

		// Call backend to update working directory by checking out the branch
		const response = await fetch(`${apiAddress}/api/check-out/branches/${input}`, {
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
	<button on:click={toggleMenu} class="branch-button" title="Set Branch Name">
		{$currentBranch.length > 100 ? `${$currentBranch.slice(0, 100)}...` : $currentBranch}
	</button>

	{#if showMenu}
		<div class="branch-menu">
			<button class="close-button" on:click={closeMenu} aria-label="Close menu">✖</button>
			<h4>Select Existing Branch</h4>
			<ul class="branch-list">
				{#each nonProtectedBranches as branch}
					<li>
						<button
							class="branch-option"
							on:click={() => setBranchName(branch)}
							on:keydown={(e) => e.key === 'Enter' && setBranchName(branch)}
							aria-label={`Select branch ${branch}`}
						>
							{branch}
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
		background-color: var(--button-background);
		color: var(--button-text);
		border: none;
		border-radius: 0.3rem;
		padding: 0.5rem 1rem;
		cursor: pointer;
		font-size: medium;
		margin-right: 1rem;
	}

	.branch-button:hover {
		background-color: var(--button-hover);
		transition: background-color 0.3s ease;
	}

	.branch-menu {
		position: absolute;
		background-color: white;
		border: 1px solid #ccc;
		padding: 1rem;
		z-index: 1000;
		min-width: 225px;
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
		color: #f00;
		cursor: pointer;
		position: absolute;
		top: 0.01rem;
		right: 0.01rem;
	}

	.branch-option {
		padding: 0.5rem 1rem;
		cursor: pointer;
		width: 100%;
		text-align: left;
		box-sizing: border-box;
	}

	.branch-option:hover {
		background-color: var(--button-hover);
	}

	input {
		margin-top: 0.5rem;
		padding: 0.5rem 0.75rem;
		border: 1px solid #ccc;
		border-radius: 0.3rem;
		width: calc(100% - 1.5rem);
		box-sizing: border-box;
	}

	button {
		margin-top: 0.5rem;
		padding: 0.5rem 1rem;
		background-color: var(--button-background);
		border: none;
		border-radius: 0.3rem;
		color: var(--button-text);
		cursor: pointer;
	}

	button:hover {
		background-color: var(--button-hover);
	}
</style>
