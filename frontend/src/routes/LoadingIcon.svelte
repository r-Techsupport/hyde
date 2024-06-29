<script lang="ts">
	export let visible: boolean = false;
</script>

{#if visible}
	<div class="background-cover"></div>
	<div class="loading-icon"></div>
{/if}

<style>
	.background-cover {
		position: absolute;
		background-color: var(--background-0);
		opacity: 0.9;
		width: 100vw;
		height: 100vh;
	}

	.loading-icon {
		position: absolute;
		left: 50%;
		top: 40%;
		width: 4rem;
		height: 4rem;
		border: 5px solid rgba(0 0 0 0);
		border-radius: 50%;
		border-top: 0.5rem solid var(--foreground-1);
		animation: spin 1.5s linear infinite;
	}

	@keyframes spin {
		0% {
			transform: rotate(0deg);
		}

		100% {
			transform: rotate(360deg);
		}
	}
</style>
