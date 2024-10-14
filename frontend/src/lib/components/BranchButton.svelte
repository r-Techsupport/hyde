<!-- BranchButton.svelte -->
<script lang="ts">
    import { branchName } from "$lib/main";
    import { derived } from 'svelte/store';

    export let initialBranchName: string; // Changed prop name for clarity

    // Create a reactive derived store to automatically update the branch name
    const currentBranch = derived(branchName, $branchName => $branchName || initialBranchName);

    function setBranchName() {
        const input = prompt("Enter the branch name:", $currentBranch); // Use derived store value

        // Define validation rules
        const maxLength = 255; // Maximum length for branch name
        const invalidCharacters = /[~^:?*<>|]/; // Invalid special characters
        const startsWithLetterOrNumber = /^[a-zA-Z0-9]/; // Starts with letter or number
        const containsSpaces = /\s/; // Contains spaces

        if (input && input !== $currentBranch) { // Check if input is different and not empty
            // Check for validation rules
            if (containsSpaces.test(input)) {
                alert("Branch names cannot contain spaces. Use dashes (-) or underscores (_) instead."); // Inform the user
            } else if (!startsWithLetterOrNumber.test(input)) {
                alert("Branch names must start with a letter or a number."); // Inform the user
            } else if (invalidCharacters.test(input)) {
                alert("Branch names cannot contain special characters like ~, ^, :, ?, *, and others."); // Inform the user
            } else if (input.length > maxLength) {
                alert(`Branch names must be shorter than ${maxLength} characters.`); // Inform the user
            } else {
                branchName.set(input); // Update the store
                console.log(`Branch name set to: ${input}`); // For debugging
            }
        } else if (!input) {
            console.log("Branch name update canceled or input is empty."); // Handle cancel or empty input
        }
    }

</script>

<button on:click={setBranchName} class="branch-button" title="Set Branch Name" aria-label="Set Branch Name">
    {$currentBranch}
</button>

<style>
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
        transition: background-color 0.3s ease; /* Add transition for a smoother effect */
    }
</style>
