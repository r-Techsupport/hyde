<script lang="ts">
	import { SelectedMode } from '$lib/main';
	export let mode;
</script>

<div class="container">
	<span
		on:click={() => {
			mode = SelectedMode.Documents;
		}}
		on:keydown={() => {
			mode = SelectedMode.Documents;
		}}
		class="label"
		role="button"
		tabindex="0"
	>
		<svg
			style="transform: rotate(90deg)"
			xmlns="http://www.w3.org/2000/svg"
			height="24px"
			viewBox="0 -960 960 960"
			width="24px"><path d="M504-480 320-664l56-56 240 240-240 240-56-56 184-184Z" /></svg
		>
		assets
	</span>
	<!-- TODO -->
	<!-- <input type="search" class="search-bar" placeholder="Filter folders..."/> -->
</div>

<style>
	.container {
		display: flex;
		flex-direction: column;
		align-items: center;
		color: var(--foreground-0);
	}

	.label {
		padding-left: 2.1rem;
		display: flex;
		cursor: pointer;
		border: none;
		background: none;
		color: var(--foreground-0);
		font-size: inherit;
		align-items: center;

		/* Sizing, spacing */
		width: 98%;
		border-radius: 5px;
		margin-left: 1%;
		margin-top: 0.35rem;
		margin-bottom: 0.4rem;
		padding-top: 0.4rem;
		padding-bottom: 0.4rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.label svg {
		fill: var(--foreground-5);
	}

	.search-bar {
		color: var(--foreground-0);
		background-color: var(--background-3);

		width: 95%;
		height: 2rem;
		border-style: none;
	}

	.search-bar::placeholder {
		padding-left: 0.5rem;
		color: var(--foreground-4);
	}
</style>
