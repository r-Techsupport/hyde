<!-- 
 This component is used as a substitute for the directory dropdown
  -->
<script lang="ts">
	export let label = '';
</script>

<span class={'container'}>
	<button on:click class="entry-button">
		<!-- Rendering if the navigation item is a directory -->
		<!-- The chevron -->
		<svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px"
			><path d="M504-480 320-664l56-56 240 240-240 240-56-56 184-184Z" /></svg
		>
		{label}
	</button>
</span>

<style>
	.entry-button {
		padding-left: 1rem;
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
		margin-top: 0.4rem;
		margin-bottom: 0.4rem;
		padding-top: 0.4rem;
		padding-bottom: 0.4rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.entry-button * {
		text-overflow: ellipsis;
		vertical-align: middle;
	}

	.container {
		display: flex;
		border-radius: 5px;
		width: 98%;
		margin: auto;
	}

	.container:hover {
		background-color: var(--background-3);
	}

	.container svg {
		fill: var(--foreground-5);
	}
</style>
