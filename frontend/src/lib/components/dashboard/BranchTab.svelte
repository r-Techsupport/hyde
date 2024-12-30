<script lang="ts">
	import { allBranches, me } from '$lib/main'; // Import your store with all branches and user data
	import { Permission } from '$lib/types';
	import SectionHeader from '../elements/SectionHeader.svelte';
	import ToggleSwitch from '../elements/ToggleSwitch.svelte';

	// Check if the user has the ManageBranches permission
	const canAccess = $me.permissions?.includes(Permission.ManageBranches);
	const sortedBranches = $allBranches.slice().sort((a, b) => a.name.localeCompare(b.name));
</script>

{#if canAccess}
	<div class="container">
		<ul>
		<SectionHeader>Enable Branch Protection</SectionHeader>
			{#each sortedBranches as branch}
				<li>
					<!-- <label class="checkbox-label">
						<input type="checkbox" bind:checked={branch.isProtected} />
						{branch.name}
					</label> -->
					<ToggleSwitch --size=1rem label="{branch.name}" />
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
		padding: 0.5rem;
	}

	ul {
		list-style-type: none;
		padding: 0;
		margin: 0;
	}

	.container li {
		white-space: nowrap;
		text-overflow: ellipsis;
		margin-top: 0.3rem;
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		width: 100%;
		height: 3rem;
		cursor: pointer;
		box-sizing: border-box;
		transition: background-color 0.3s ease;
	}

	.checkbox-label input {
		margin-right: 10px;
	}

	.checkbox-label:hover {
		background-color: var(--background-3);
	}

	.checkbox-label input:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}

	p {
		color: var(--foreground-2);
		text-align: center;
		margin-top: 2rem;
		font-size: 1.2rem;
	}
</style>
