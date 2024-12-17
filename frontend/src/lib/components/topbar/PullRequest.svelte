<!-- PullRequest.svelte -->
<script lang="ts">
    import { onMount } from 'svelte';
    import { ToastType, addToast } from '$lib/toast';
    import { apiAddress, branchName, currentFile, me, openIssues, selectedIssues } from '$lib/main';
    import type { Issue } from '$lib/types';
    import { get } from 'svelte/store';

    let showModal = false;
    let selectedIssueDetails: Issue | null = null;

    function openModal() {
        showModal = true;

        document.addEventListener('keydown', handleEscape);
    }

    function closeModal() {
        selectedIssues.set([]);
        showModal = false;
        selectedIssueDetails = null;

        document.removeEventListener('keydown', handleEscape);
    }

    function handleEscape(event: KeyboardEvent) {
        if (event.key === 'Escape') {
            closeModal();
        }
    }

    function toggleSelection(issue: Issue) {
    selectedIssues.update((issues: Issue[]) => {
        if (issues.includes(issue)) {
            return issues.filter(i => i !== issue);
        } else {
            return [...issues, issue];
        }
    });
}
    onMount(() => {
        getOpenIssues();
    });

    async function getOpenIssues() {
        const state = "open";
        const labels = "";
        const url = `${apiAddress}/api/issues/${state}${labels ? `?labels=${labels}` : ''}`;

        try {
            // Fetch the data from the API
            const response = await fetch(url, {
                method: 'GET',
                credentials: 'include',
            });

            // Check if the response is successful (status code 2xx)
            if (!response.ok) {
                const errorMessage = `Failed to fetch open issues. (Code ${response.status}: "${response.statusText}")`;
                addToast({
                    message: errorMessage,
                    type: ToastType.Error,
                    dismissible: true,
                });
                return;
            }

            // Parse the response as JSON
            const responseData = await response.json();

            // Validate the response structure
            if (responseData.status === "success" && Array.isArray(responseData.data?.issues)) {
                const issuesOnly = responseData.data.issues.filter((issue: Issue) => !issue.pull_request);
                openIssues.set(issuesOnly);
                console.log($openIssues);  // Optional: for debugging purposes
            } else {
                // Handle unexpected response structure
                const errorMessage = `Unexpected response structure: ${JSON.stringify(responseData)}`;
                addToast({
                    message: errorMessage,
                    type: ToastType.Error,
                    dismissible: true,
                });
            }
        } catch (error: unknown) {
            // Handle fetch or network errors
            let errorMessage = 'An unknown error occurred.';
            if (error instanceof Error) {
                errorMessage = `An error occurred while processing the response: ${error.message}`;
            }

            addToast({
                message: errorMessage,
                type: ToastType.Error,
                dismissible: true,
            });
        }
    }

    let createPullRequestHandler = async (): Promise<void> => {
		const title = `Pull request for ${$currentFile}`;
		const description = `This pull request contains changes made by ${$me.username}.`;
		const headBranch = $branchName;

        // Get selected issues from the store
        const selectedIssueNumbers = get(selectedIssues).map((issue: Issue) => issue.number);
        console.log(selectedIssueNumbers)

		const response = await fetch(`${apiAddress}/api/pulls`, {
			method: 'POST',
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				head_branch: headBranch,
				base_branch: 'master',
				title: title,
				description: description,
                issue_numbers: selectedIssueNumbers
			})
		});

		// Handle the response
		if (!response.ok) {
			const errorMessage = `Failed to create pull request (Code ${response.status}: "${response.statusText}")`;
			addToast({
				message: `Error: ${errorMessage}`,
				type: ToastType.Error,
				dismissible: true
			});
			return; // Exit the function early on error
		}

		// Parse the JSON response to get the pull request URL
		const jsonResponse = await response.json();
		const pullRequestUrl = jsonResponse.data?.pull_request_url; // Adjusted based on API response

		if (pullRequestUrl) {
			// If successful, show success toast with the URL
			addToast({
				message: `Pull request created successfully. View it [here](${pullRequestUrl}).`,
				type: ToastType.Success,
				dismissible: true
			});
		} else {
			// Handle the case where the URL is not present (if needed)
			addToast({
				message: 'Pull request created successfully, but the URL is not available.',
				type: ToastType.Warning,
				dismissible: true
			});
		}
	};

</script>

<div class="pull-request">
    <!-- Pull Request -->
	<button on:click={openModal} class="pull-request" title="Pull Request" type="button">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="25px"
			height="25px"
			viewBox="0 0 24 24"
			fill="none"
		>
			<path
				fill-rule="evenodd"
				clip-rule="evenodd"
				d="M14.7071 2.70711L13.4142 4H14C17.3137 4 20 6.68629 20 10V16.1707C21.1652 16.5825 22 17.6938 22 19C22 20.6569 20.6569 22 19 22C17.3431 22 16 20.6569 16 19C16 17.6938 16.8348 16.5825 18 16.1707V10C18 7.79086 16.2091 6 14 6H13.4142L14.7071 7.29289C15.0976 7.68342 15.0976 8.31658 14.7071 8.70711C14.3166 9.09763 13.6834 9.09763 13.2929 8.70711L10.2929 5.70711C9.90237 5.31658 9.90237 4.68342 10.2929 4.29289L13.2929 1.29289C13.6834 0.902369 14.3166 0.902369 14.7071 1.29289C15.0976 1.68342 15.0976 2.31658 14.7071 2.70711ZM18 19C18 18.4477 18.4477 18 19 18C19.5523 18 20 18.4477 20 19C20 19.5523 19.5523 20 19 20C18.4477 20 18 19.5523 18 19ZM6 4C5.44772 4 5 4.44772 5 5C5 5.55228 5.44772 6 6 6C6.55228 6 7 5.55228 7 5C7 4.44772 6.55228 4 6 4ZM7 7.82929C8.16519 7.41746 9 6.30622 9 5C9 3.34315 7.65685 2 6 2C4.34315 2 3 3.34315 3 5C3 6.30622 3.83481 7.41746 5 7.82929V16.1707C3.83481 16.5825 3 17.6938 3 19C3 20.6569 4.34315 22 6 22C7.65685 22 9 20.6569 9 19C9 17.6938 8.16519 16.5825 7 16.1707V7.82929ZM6 18C5.44772 18 5 18.4477 5 19C5 19.5523 5.44772 20 6 20C6.55228 20 7 19.5523 7 19C7 18.4477 6.55228 18 6 18Z"
				fill="#bababa"
			/>
		</svg>
		<span>Create Pull Request</span>
	</button>

    <!-- Modal Implementation -->
    {#if showModal}
        <div class="modal-backdrop">
            <div class="modal-content">
                <h2>Open Issues</h2>
                <button class="close-btn" on:click={closeModal} type="button" aria-label="Close modal">
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
                <div>
                    <ul>
                        {#each $openIssues as issue}
                            <li>
                                <button on:click={() => selectedIssueDetails = issue} class="issue-title">
                                    {issue.title}
                                </button>
                                {#if selectedIssueDetails === issue}
                                    <p>{issue.body}</p>
                                {/if}
                                <button on:click={() => toggleSelection(issue)}>
                                    {#if $selectedIssues.includes(issue)}
                                        Deselect
                                    {:else}
                                        Select
                                    {/if}
                                </button>
                            </li>
                        {/each}
                    </ul>
                    <p>Selected Issues: {$selectedIssues.length}</p>

                    <!-- Submit Pull Request Button -->
                    <button on:click={createPullRequestHandler} class="submit-pr-btn">
                        Submit Pull Request
                    </button>
                </div>
            </div>
        </div>
    {/if}
</div>

<style>
    .pull-request {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        background: transparent;
        border: none;
        padding: 0.125rem 0.25rem;
        color: var(--foreground-3);
        font-size: 1.25rem;
        margin-right: 1rem;
    }

    .pull-request:hover {
		background-color: var(--background-1);
		transition: background-color 0.3s ease;
	}

    .pull-request span {
        margin-left: 0.25rem;
    }

    .pull-request svg {
        margin-top: 0.125rem;
    }

    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        outline: none;
    }

    .modal-content {
        background: white;
        padding: 1.5rem;
        border-radius: 8px;
        position: relative;
    }

    .close-btn {
        background: none;
        border: none;
        font-size: 1.5rem;
        cursor: pointer;
    }

    .issue-title {
        background: none;
        border: none;
        text-decoration: underline;
        cursor: pointer;
    }

    ul {
        list-style-type: none;
        padding: 0;
    }

    li {
        margin-bottom: 10px;
    }

    .submit-pr-btn {
        margin-top: 1rem;
        padding: 0.5rem 1rem;
        background-color: var(--foreground-3);
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 1rem;
        width: 100%;
    }

    .submit-pr-btn:hover {
        background-color: var(--foreground-5);
    }
</style>