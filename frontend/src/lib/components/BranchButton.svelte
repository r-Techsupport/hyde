<!-- BranchButton.svelte -->
<script lang="ts">
	import { branchName } from '$lib/main';
	import { derived } from 'svelte/store';
	import { onMount } from 'svelte';
	import { apiAddress } from '$lib/net';
	import { ToastType, addToast } from '$lib/toast';

	let showMenu = false;
	let existingBranches: string[] = [];
	let newBranchName: string = '';

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
				alert(`Error fetching branches: ${response.statusText}. ${JSON.stringify(errorMessage)}`);
			});
			return; // Exit if response is not OK
		}

		// Extract and set the branches if the response is successful
		response.json().then((data) => {
			if (data.data && data.data.branches) {
				existingBranches = data.data.branches;
			} else {
				existingBranches = []; // Reset if no branches found
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

		// Set branch name and reset state
		branchName.set(input);
		newBranchName = '';
		showMenu = false;

		// Call backend to update working directory by checking out the branch
		try {
			const response = await fetch(`${apiAddress}/api/check-out/branches/${input}`, {
				method: 'PUT',
				credentials: 'include'
			});

			console.log(response);

			if (!response.ok) {
				throw new Error(`Failed to check out branch. Error ${response.status}: ${response.statusText}`);
			}

			// Successfully checked out branch
			console.log('Branch checked out:', await response.text());
			await fetchExistingBranches();

		} catch (error) {
			addToast({
				message: `Failed to check out branch "${input}". Unknown error occurred.`,
				type: ToastType.Error,
				dismissible: true
			});
		}
	}

	function toggleMenu() {
		showMenu = !showMenu;
	}

	function closeMenu() {
		showMenu = false;
	}
</script>

<div class="branch-dropdown">
	<button
		on:click={toggleMenu}
		class="branch-button"
		title="Set Branch Name"
		aria-label="Set Branch Name"
	>
		{$currentBranch}
	</button>

	{#if showMenu}
		<div class="branch-menu">
			<button class="close-button" on:click={closeMenu} aria-label="Close menu">✖</button>
			<h4>Select Existing Branch</h4>
			<ul class="branch-list">
				{#each existingBranches as branch}
					<li>
						<button
							class="branch-option"
							on:click={() => setBranchName(branch)}
							on:keydown={(e) => e.key === 'Enter' && setBranchName(branch)}
							aria-label={`Select branch ${branch}`}
						>
							{branch}
							<!-- This line will display the branch name -->
						</button>
					</li>
				{/each}
				{#if existingBranches.length === 0}
					<li>No branches available</li>
					<!-- Show message if no branches -->
				{/if}
			</ul>
			<h4>Create New Branch</h4>
			<input
				type="text"
				bind:value={newBranchName}
				placeholder="Enter new branch name"
				on:keydown={(e) => e.key === 'Enter' && setBranchName(newBranchName)}
			/>
			<button on:click={() => setBranchName(newBranchName)}>Create Branch</button>
		</div>
	{/if}
</div>

<style>
	.branch-dropdown {
		position: relative;
	}

	.branch-button {
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
	}

	.close-button {
		background: none;
		border: none;
		color: #f00;
		cursor: pointer;
		font-size: 1.2rem;
		position: absolute;
		top: 0.5rem;
		right: 0.5rem;
	}

	.branch-option {
		padding: 0.5rem;
		cursor: pointer;
	}

	.branch-option:hover {
		background-color: var(--button-hover);
	}

	input {
		margin-top: 0.5rem;
		padding: 0.5rem;
		border: 1px solid #ccc;
		border-radius: 0.3rem;
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
