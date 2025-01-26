<script lang="ts">
	import { apiAddress } from '$lib/main';
	import { addToast, dismissToast, ToastType } from '$lib/toast';
	import SectionHeader from '../elements/SectionHeader.svelte';
</script>

<div class="container">
	<!-- Actions -->
	<SectionHeader>Actions</SectionHeader>
	<!-- Admin Dashboard -->
	<div>
		<button
			onclick={async () => {
				const toastId = addToast(
					`Cloning fresh repository, this may take a while...`,
					ToastType.Info,
					false
				);
				const response = await fetch(`${apiAddress}/api/reclone`, {
					method: 'POST',
					credentials: 'include'
				});
				dismissToast(toastId);
				if (response.ok) {
					addToast(`Cloned fresh repository successfully`, ToastType.Success);
				} else {
					addToast(`Clone failed, check server logs`, ToastType.Error);
				}
			}}
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				height="24px"
				viewBox="0 -960 960 960"
				width="24px"
				fill="#e8eaed"
				><path
					d="M160-160v-80h110l-16-14q-52-46-73-105t-21-119q0-111 66.5-197.5T400-790v84q-72 26-116 88.5T240-478q0 45 17 87.5t53 78.5l10 10v-98h80v240H160Zm400-10v-84q72-26 116-88.5T720-482q0-45-17-87.5T650-648l-10-10v98h-80v-240h240v80H690l16 14q49 49 71.5 106.5T800-482q0 111-66.5 197.5T560-170Z"
				/></svg
			>
			Reclone Repository
		</button>
	</div>
	<!-- Metrics -->
	<SectionHeader>Metrics</SectionHeader>
	TODO
</div>

<style>
	.container {
		margin: 0.3rem;
		width: 100%;
	}

	.container * {
		font-size: 1rem;
		color: var(--foreground-3);
	}

	button {
		display: flex;
		align-items: center;
		padding-left: 1rem;
		width: 100%;
		background-color: transparent;
		border: none;
	}

	button:hover {
		background-color: var(--background-2);
		cursor: pointer;
	}

	svg {
		fill: var(--foreground-3);
		padding-right: 0.2rem;
	}
</style>
