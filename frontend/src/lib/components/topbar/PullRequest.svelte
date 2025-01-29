<!-- PullRequest.svelte -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { ToastType, addToast } from '$lib/toast';
	import { apiAddress, branchName, currentFile, me, baseBranch } from '$lib/main';
	import type { Issue } from '$lib/types';
	import LoadingIcon from '../elements/LoadingIcon.svelte';

	let showModal = false;
	let prCommit = '';
	let isExpanded: { [key: number]: boolean } = {};
	let showLoadingIcon: boolean;
	let selectedPullRequest: number | null = null;
	let prAuthor = '';
	let openIssues: Issue[] = [];
	let selectedIssues: Issue[] = [];
	let openPullRequests: Issue[] = [];

	function openModal() {
		showModal = true;
		checkOpenPullRequests();
		document.addEventListener('keydown', handleEscape);
	}

	function closeModal() {
		selectedIssues = [];
		prCommit = '';
		showModal = false;
		selectedPullRequest = null;
		showLoadingIcon = false;

		document.removeEventListener('keydown', handleEscape);
	}

	function toggleExpand(issueId: number) {
		isExpanded[issueId] = !isExpanded[issueId];
	}

	function handleEscape(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			closeModal();
		}
	}

	function toggleSelection(issue: Issue): void {
		const idx = selectedIssues.findIndex((i) => i.id === issue.id);
		if (idx !== -1) {
			selectedIssues = selectedIssues.filter((i) => i.id !== issue.id);
		} else {
			selectedIssues = [...selectedIssues, issue];
		}
	}

	onMount(() => {
		getOpenIssues();
	});

	async function getOpenIssues(): Promise<void> {
		const state = 'open';
		const labels = '';
		const url = `${apiAddress}/api/issues/${state}${labels ? `?labels=${labels}` : ''}`;

		// Fetch the data from the API
		const response = await fetch(url, {
			method: 'GET',
			credentials: 'include'
		});

		// Check if the response is successful (status code 2xx)
		if (!response.ok) {
			const errorMessage = `Failed to fetch open issues. (Code ${response.status}: "${response.statusText}")`;
			addToast(errorMessage, ToastType.Error);
			return;
		}

		// Parse the response as JSON
		const responseData = await response.json();
		const issues = responseData.issues ?? [];

		const issuesOnly = issues.filter((issue: Issue) => !issue.pull_request);
		const pullRequestsOnly = issues.filter((issue: Issue) => issue.pull_request);
		openIssues = issuesOnly;
		openPullRequests = pullRequestsOnly;
	}

	async function createPullRequest(): Promise<void> {
		const title = `Pull request form: ${$me.username}`;
		const prDescription = `Changes made from ${$currentFile}.\n ${prCommit}`;
		const headBranch = $branchName;

		const selectedIssueNumbers = selectedIssues.map((issue: Issue) => issue.number);
		showLoadingIcon = true;

		const response = await fetch(`${apiAddress}/api/pulls`, {
			method: 'POST',
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				head_branch: headBranch,
				base_branch: $baseBranch,
				title: title,
				description: prDescription,
				issue_numbers: selectedIssueNumbers
			})
		});

		// Handle the response
		if (!response.ok) {
			const errorMessage = `Failed to create pull request (Code ${response.status}: "${response.statusText}")`;
			addToast(`Error: ${errorMessage}`, ToastType.Error);
			return;
		}

		// Parse the JSON response to get the pull request URL
		const jsonResponse = await response.json();
		const pullRequestUrl = jsonResponse.pull_request_url;

		if (pullRequestUrl) {
			// If successful, show success toast with the URL
			addToast(
				`Pull request created successfully. View it here: ${pullRequestUrl}`,
				ToastType.Success
			);
		} else {
			// Handle the case where the URL is not present (if needed)
			addToast(
				'Pull request created successfully, but the URL is not available.',
				ToastType.Warning
			);
		}
		showLoadingIcon = false;
		closeModal();
	}

	async function updatePullRequest(): Promise<void> {
		// Ensure the current user is the PR author
		if ($me.groups?.some((group) => group.name === 'Admin') || prAuthor === $me.username) {
			const title = `Updated pull request form: ${$me.username}`;
			const pr_description = `Updated changes made from ${$currentFile}.\n ${prCommit}`;

			const selectedIssueNumbers = selectedIssues.map((issue: Issue) => issue.number);
			showLoadingIcon = true;

			const response = await fetch(`${apiAddress}/api/pulls/update`, {
				method: 'PUT',
				credentials: 'include',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					pr_number: selectedPullRequest,
					title: title,
					description: pr_description,
					base_branch: $baseBranch,
					issue_numbers: selectedIssueNumbers
				})
			});

			// Handle the response
			if (!response.ok) {
				const errorMessage = `Failed to update pull request (Code ${response.status}: "${response.statusText}")`;
				addToast(`Error: ${errorMessage}`, ToastType.Error, true);
				showLoadingIcon = false;
				return;
			}

			// Parse the JSON response to get the updated pull request URL or other details
			const pullRequestUrl = await response.json();

			if (pullRequestUrl) {
				// If successful, show success toast with the URL
				addToast(
					`Pull request updated successfully. View it here: ${pullRequestUrl}`,
					ToastType.Success
				);
			} else {
				// Handle the case where the URL is not present (if needed)
				addToast(
					'Pull request updated successfully, but the URL is not available.',
					ToastType.Warning
				);
			}
		} else {
			// If the user is not an admin and not the PR author, deny deletion
			addToast(`Error: You are not authorized to delete this pull request.`, ToastType.Error);
			return;
		}
		showLoadingIcon = false;
		closeModal();
	}

	async function closePullRequest(): Promise<void> {
		// Check if the current user is the PR author
		if ($me.groups?.some((group) => group.name === 'Admin') || prAuthor === $me.username) {
			showLoadingIcon = true;

			const response = await fetch(`${apiAddress}/api/pull-requests/${selectedPullRequest}/close`, {
				method: 'POST',
				credentials: 'include'
			});

			if (!response.ok) {
				const errorMessage = `Failed to delete close request (Code ${response.status}: "${response.statusText}")`;
				addToast(`Error: ${errorMessage}`, ToastType.Error);
				showLoadingIcon = false;
				return;
			}

			const jsonResponse = await response.json();
			addToast(jsonResponse, ToastType.Success);
			closeModal();
		} else {
			// If the user is not an admin and not the PR author, deny deletion
			addToast('Error: You are not authorized to close this pull request.', ToastType.Error);
			return;
		}

		showLoadingIcon = false;
	}

	async function checkOpenPullRequests() {
		showLoadingIcon = true;
		// Loop through each open pull request
		for (const pr of openPullRequests) {
			if (!pr.pull_request) continue;

			// Fetch the details of the pull request using the prNumber
			const response = await fetch(pr.pull_request.url);
			if (!response.ok) {
				continue;
			}

			const prData = await response.json();
			prAuthor = pr.title.split(':')[1]?.trim();
			const sourceBranch = prData.head.ref;

			// Compare the source branch with the current branch
			if (sourceBranch === $branchName) {
				// Extract issue numbers from the PR body (e.g., #25, #28)
				const issueRegex = /#(\d+)/g;
				const linkedIssues: number[] = [];
				let match;
				while ((match = issueRegex.exec(pr.body)) !== null) {
					linkedIssues.push(parseInt(match[1], 10));
				}
				linkedIssues.forEach((issueNumber) => {
					const matchingIssue = openIssues.find((issue) => issue.number === issueNumber);
					if (matchingIssue) {
						toggleSelection(matchingIssue);
					}
				});
				prCommit = pr.body.replace(/Closes\s+#?\d+/g, '').trim();
				selectedPullRequest = pr.number;
				break;
			}
		}
		showLoadingIcon = false;
	}
</script>

<div class="pull-request">
	<!-- Pull Request -->
	<button on:click={() => openModal()} class="pull-request" title="Pull Request" type="button">
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
				<button
					class="close-btn"
					on:click={() => closeModal()}
					type="button"
					aria-label="Close modal"
				>
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
					<!-- Checkbox Group -->
					<ul>
						{#each openIssues as issue}
							<li>
								<div class="issues">
									<!-- Checkbox -->
									<input
										type="checkbox"
										id={`issue-${issue.id}`}
										checked={selectedIssues.includes(issue)}
										on:change={() => toggleSelection(issue)}
									/>
									<!-- Label and Issue Title -->
									<div class="issue-container">
										<label for={`issue-${issue.id}`}>
											<span class="issue-title">
												{issue.title}
												<!-- SVG Icon for the Issue URL -->
												<a
													href={issue.html_url}
													aria-label="Issue SVG"
													target="_blank"
													rel="noopener noreferrer"
													class="issue-svg"
												>
													<svg viewBox="0 0 22 22" xmlns="http://www.w3.org/2000/svg">
														<path
															fill-rule="evenodd"
															d="M5,2 L7,2 C7.55228475,2 8,2.44771525 8,3 C8,3.51283584 7.61395981,3.93550716 7.11662113,3.99327227 L7,4 L5,4 C4.48716416,4 4.06449284,4.38604019 4.00672773,4.88337887 L4,5 L4,19 C4,19.5128358 4.38604019,19.9355072 4.88337887,19.9932723 L5,20 L19,20 C19.5128358,20 19.9355072,19.6139598 19.9932723,19.1166211 L20,19 L20,17 C20,16.4477153 20.4477153,16 21,16 C21.5128358,16 21.9355072,16.3860402 21.9932723,16.8833789 L22,17 L22,19 C22,20.5976809 20.75108,21.9036609 19.1762728,21.9949073 L19,22 L5,22 C3.40231912,22 2.09633912,20.75108 2.00509269,19.1762728 L2,19 L2,5 C2,3.40231912 3.24891996,2.09633912 4.82372721,2.00509269 L5,2 L7,2 L5,2 Z M21,2 L21.081,2.003 L21.2007258,2.02024007 L21.2007258,2.02024007 L21.3121425,2.04973809 L21.3121425,2.04973809 L21.4232215,2.09367336 L21.5207088,2.14599545 L21.5207088,2.14599545 L21.6167501,2.21278596 L21.7071068,2.29289322 L21.7071068,2.29289322 L21.8036654,2.40469339 L21.8036654,2.40469339 L21.8753288,2.5159379 L21.9063462,2.57690085 L21.9063462,2.57690085 L21.9401141,2.65834962 L21.9401141,2.65834962 L21.9641549,2.73400703 L21.9641549,2.73400703 L21.9930928,2.8819045 L21.9930928,2.8819045 L22,3 L22,3 L22,9 C22,9.55228475 21.5522847,10 21,10 C20.4477153,10 20,9.55228475 20,9 L20,5.414 L13.7071068,11.7071068 C13.3466228,12.0675907 12.7793918,12.0953203 12.3871006,11.7902954 L12.2928932,11.7071068 C11.9324093,11.3466228 11.9046797,10.7793918 12.2097046,10.3871006 L12.2928932,10.2928932 L18.584,4 L15,4 C14.4477153,4 14,3.55228475 14,3 C14,2.44771525 14.4477153,2 15,2 L21,2 Z"
														></path>
													</svg>
												</a>
											</span>
										</label>
										<!-- Display Issue Body with Show More/Show Less -->
										<div class="issue-body">
											{#if issue.body.length > 200}
												<p>
													{#if !isExpanded[issue.id]}
														{issue.body.slice(0, 200)}...
													{:else}
														{issue.body}
													{/if}
												</p>
												<button on:click={() => toggleExpand(issue.id)} class="show-more-button">
													{#if !isExpanded[issue.id]}
														Show More
													{:else}
														Show Less
													{/if}
												</button>
											{:else}
												<p>{issue.body}</p>
											{/if}
										</div>
									</div>
								</div>
							</li>
						{/each}
					</ul>
					<textarea
						bind:value={prCommit}
						placeholder="Enter pull request description"
						rows="5"
						class="description-textarea"
					></textarea>
					<!-- Pull Request Button -->
					{#if selectedPullRequest === null}
						<button on:click={createPullRequest} class="submit-pr-btn">
							Submit Pull Request
						</button>
					{:else}
						<button on:click={updatePullRequest} class="submit-pr-btn">
							Update Pull Request
						</button>
						<button on:click={closePullRequest} class="submit-pr-btn"> Delete Pull Request </button>
					{/if}
				</div>
			</div>
		</div>
	{/if}
</div>
<LoadingIcon bind:visible={showLoadingIcon} />

<style>
	.pull-request {
		display: flex;
		box-sizing: content-box;
		height: inherit;
		align-items: center;
		justify-content: flex-end;
		background: transparent;
		border: none;
		color: var(--foreground-3);
		font-size: 1.25rem;
		border-radius: 5px;
		padding: 0.2rem;
		margin-right: 1rem;
	}

	.pull-request span {
		margin-left: 0.25rem;
		text-overflow: clip;
		white-space: nowrap;
	}

	.pull-request > span:hover {
		background-color: var(--background-1);
		transition: background-color 0.3s ease;
	}

	.pull-request > svg {
		margin-top: 0.125rem;
		min-height: 20px;
		min-width: 20px;
	}

	.modal-backdrop {
		position: fixed;
		top: 0;
		left: 0;
		width: 100vw;
		height: 100vh;
		display: flex;
		justify-content: center;
		align-items: center;
		outline: none;
	}

	.modal-content {
		position: absolute;
		background: var(--background-2);
		padding: 1.5rem;
		border-radius: 8px;
		width: 60%;
		max-height: 60%;
		margin-left: 1rem;
		overflow-y: scroll;
		z-index: 1000;
	}

	.modal-content h2 {
		margin: 0;
	}

	.close-btn {
		position: absolute;
		top: 0.5rem;
		right: 0.5rem;
		background: none;
		border: none;
		font-size: 1.5rem;
		cursor: pointer;
		fill: var(--foreground-3);
		transition: color 0.2s ease-in-out;
	}

	.issues {
		display: flex;
		align-items: flex-start;
		gap: 0.15rem;
		margin-left: -2.75rem;
	}

	.issue-container {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
	}

	.issues input[type='checkbox'] {
		min-width: 1.25rem;
		min-height: 1.25rem;
		cursor: pointer;
		accent-color: var(--background-1);
	}

	.issues label {
		display: flex;
		align-items: center;
		cursor: pointer;
	}

	.issue-title {
		display: flex;
		width: 100%;
		flex-direction: row;
		background: none;
		border: none;
		font-size: 1rem;
		font-weight: 500;
		color: var(--foreground-1);
		cursor: pointer;
		transition: color 0.2s ease-in-out;
	}

	.issue-title a {
		height: 1rem;
		width: 1rem;
		color: var(--foreground-1);
		text-decoration: none;
		margin-left: 0.3rem;
	}

	.issue-svg {
		max-width: 0.9rem;
		max-height: 0.9rem;
		margin-top: 0;
		fill: var(--foreground-1);
		transition: fill 0.2s ease-in-out;
	}

	.issue-body {
		font-size: 0.75rem;
		color: var(--foreground-3);
		line-height: 1.5;
	}

	.issue-body p {
		margin: 0;
	}

	.show-more-button {
		font-size: 0.6rem;
		padding: 0.1rem 0.3rem;
		background-color: transparent;
		border: none;
		border-radius: 0.375rem;
		color: var(--foreground-3);
		cursor: pointer;
		transition:
			background-color 0.2s ease-in-out,
			color 0.2s ease-in-out;
	}

	.show-more-button:hover {
		background-color: var(--background-2);
		color: var(--primary);
	}

	.submit-pr-btn {
		margin-top: 1rem;
		padding: 0.5rem 1rem;
		background-color: var(--background-4);
		color: var(--foreground-1);
		border: none;
		border-radius: 4px;
		cursor: pointer;
		font-size: 1rem;
		width: 100%;
	}

	.submit-pr-btn:hover {
		background-color: var(--foreground-5);
	}

	.description-textarea {
		width: 100%;
		margin-top: 1rem;
		padding: 0.5rem;
		resize: vertical;
		outline: none !important;
		background: var(--background-3);
		color: var(--foreground-0);
	}

	ul {
		list-style: none;
	}

	li {
		margin-bottom: 1rem;
	}
</style>
