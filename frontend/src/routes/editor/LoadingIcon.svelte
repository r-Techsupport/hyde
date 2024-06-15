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
		opacity: 90%;
		width: 100vw;
		height: 100vh;
	}

	.loading-icon {
		position: absolute;
		left: 50%;
		top: 40%;
		width: 4rem;
		height: 4rem;
		/* margin: -4rem 0 0 8rem; */
		border: 0.5rem solid rgba(0, 0, 0, 0);
		border-radius: 50%;
		border-top: 0.5rem solid var(--foreground-1);
		-webkit-animation: spin 1s linear infinite;
		animation: spin 1s linear infinite;
	}

	@-webkit-keyframes spin {
		0% {
			-webkit-transform: rotate(0deg);
		}
		100% {
			-webkit-transform: rotate(360deg);
		}
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
