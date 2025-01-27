<script lang="ts">
	interface Props {
		visible: boolean;
	}

	let { visible = $bindable() }: Props = $props();
</script>

{#if visible}
	<div class="background-cover"></div>
	<div class="loading-icon"></div>
{/if}

<style>
	.background-cover {
		position: absolute;
		left: 0;
		top: 0;
		background-color: rgba(0 0 0 / 90%);
		backdrop-filter: blur(5px);
		z-index: 10;
		width: 100vw;
		height: 100vh;
	}

	.loading-icon {
		position: absolute;
		z-index: 11;
		left: 50%;
		top: 40%;
		width: 4rem;
		height: 4rem;
		border-radius: 50%;
		border: 0.5rem solid transparent;
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
