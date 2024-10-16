<!-- BranchButton.svelte -->
<script lang="ts">
    import { branchName } from '$lib/main';
    import { derived } from 'svelte/store';
    import { onMount } from 'svelte';
	import { apiAddress } from '$lib/net';

    export let initialBranchName: string;
    let showMenu = false;
    let existingBranches: string[] = [];
    let newBranchName: string = '';

    const currentBranch = derived(branchName, ($branchName) => $branchName || initialBranchName);

    async function fetchExistingBranches() {
		const response = await fetch(`${apiAddress}/api/branches`, {
			method: 'GET',
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json',
			},
		});

		// Check if the response is successful
		if (!response.ok) {
        const errorMessage = await response.json();
        throw new Error(`Error fetching branches: ${response.statusText}. ${JSON.stringify(errorMessage)}`);
    }

		existingBranches = await response.json();
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
            "Branch names must start with a letter or a number.",
            "Branch names cannot contain spaces. Use dashes (-) or underscores (_) instead.",
            "Branch names cannot contain special characters like ~, ^, :, ?, *, and others.",
            `Branch names must be shorter than ${maxLength} characters.`
        ];

        let isValid = true;

        if (input) {
            if (containsSpaces.test(input) || !startsWithLetterOrNumber.test(input) || 
                invalidCharacters.test(input) || input.length > maxLength) {
                isValid = false;
            }

            if (!isValid) {
                alert("Please ensure your branch name follows these rules:\n\n" + rules.join("\n"));
            } else {
                branchName.set(input);
                newBranchName = '';
                showMenu = false;
				await fetchExistingBranches();
            }
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
                            {branch}  <!-- This line will display the branch name -->
                        </button>
                    </li>
                {/each}
                {#if existingBranches.length === 0}
                    <li>No branches available</li> <!-- Show message if no branches -->
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
        color: #ff0000;
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
