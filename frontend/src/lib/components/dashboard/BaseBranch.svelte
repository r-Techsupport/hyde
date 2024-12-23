<script lang="ts">
	import { baseBranch, allBranches } from '$lib/main';
</script>

<div>
	<h2 class="heading">Current Base Branch: {$baseBranch}</h2>

	<!-- Dropdown to select a branch -->
	<select bind:value={$baseBranch} class="branch-dropdown">
		{#each $allBranches as { name, isProtected }}
			<option value={name}>{name}</option>
		{/each}
	</select>
</div>

<style>
	.heading {
		margin-left: 1.5rem;
		font-size: 1.25rem;
		font-weight: bold;
	}

	.branch-dropdown {
		font-size: 1rem;
		padding: 0.5rem;
		width: 15rem;
		height: 2.5rem;
		background-color: var(--background-1);
		border-radius: 0.25rem;
		margin-left: 1.5rem;
		color: var(--foreground-0);
	}

	.branch-dropdown option {
		padding: 0.625rem;
	}
</style>
