<script lang="ts">
	import { baseBranch, allBranches, me } from '$lib/main'; // Import your store with all branches and user data
	import { Permission } from '$lib/types';
	import SectionHeader from '../elements/SectionHeader.svelte';
	import ToggleSwitch from '../elements/ToggleSwitch.svelte';
	
	// Check if the user has the ManageBranches permission
	const canAccess = $me.permissions?.includes(Permission.ManageBranches);
	const sortedBranches = $allBranches.slice().sort((a, b) => a.name.localeCompare(b.name));
</script>

{#if canAccess}
	<div class="container">
		<SectionHeader --scale="0.9rem" --text-color="var(--foreground-2)">Pull Requests</SectionHeader>
		<div class="base-branch-container">
			<label for="base-branch">Destination</label>
			<select name="base-branch" bind:value={$baseBranch} class="branch-dropdown">
				{#each $allBranches as { name }}
					<option value={name}>{name}</option>
				{/each}
			</select>
			<p>All pull requests opened by Hyde will pull into this branch.</p>
		</div>
		<ul>
		<SectionHeader --scale="0.9rem">Enable Branch Protection For:</SectionHeader>
			{#each sortedBranches as branch}
				<li>
					<!-- <label class="checkbox-label">
						<input type="checkbox" bind:checked={branch.isProtected} />
						{branch.name}
					</label> -->
					<ToggleSwitch --size="0.97rem" bind:checked={branch.isProtected}>{branch.name}</ToggleSwitch>
				</li>
			{/each}
		</ul>
	</div>
{:else}
	<p>You do not have permission to view this page.</p>
{/if}

<style>
	.container {
		display: flex;
		flex-direction: column;
		width: 100%;
		max-height: 100%;
		overflow-y: scroll;
		justify-items: center;
		padding: 0.5rem 1rem 0.5rem 1rem;
	}

	.container ul {
		list-style-type: none;
		padding: 0;
		margin: 0;
	}

	.container li {
		white-space: nowrap;
		text-overflow: ellipsis;
		margin-top: 0.3rem;
	}

	.base-branch-container {
		display: flex;
		flex-direction: column;
		margin: 0.3rem;
	}
	
	.base-branch-container label {
		margin-left: 0.1rem;
		font-size: 0.8rem;
		color: var(--foreground-3);
		width: 100%;
	}

	.base-branch-container p {
		color: var(--foreground-4);
		font-size: 0.6rem;
		margin: 0.1rem;

		margin-left: 0.3rem;
	}

	.branch-dropdown {
		font-size: 1rem;
		padding: 0.25rem;
		height: 2rem;
		margin: 0.1rem;
		width: 100%;
		max-width: 20rem;
		border-radius: 3px;
		background-color: var(--background-1);
		color: var(--foreground-0);
	}

	.branch-dropdown option {
		padding: 0.625rem;
	}
</style>
