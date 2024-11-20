<script lang="ts">
	import { apiAddress } from '$lib/main';
	import { addToast, dismissToast, ToastType } from '$lib/toast';
</script>

<div class="container">
	<!-- Actions -->
	<div class="header">
		<p>Actions</p>
		<hr />
	</div>
	<!-- Admin Dashboard -->
	<div>
		<button
			onclick={async () => {
				const tid = addToast({
					message: `Cloning fresh repository, this may take a while...`,
					type: ToastType.Info,
					dismissible: false
				});
				const response = await fetch(`${apiAddress}/api/reclone`, {
					method: 'POST',
					credentials: 'include'
				});
				dismissToast(tid);
				if (response.ok) {
					addToast({
						message: `Cloned fresh repository successfully`,
						type: ToastType.Success,
						dismissible: true,
						timeout: 3000
					});
				} else {
					addToast({
						message: `Clone failed, check server logs`,
						type: ToastType.Error,
						dismissible: true,
						timeout: 3000
					});
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
	<div class="header">
		<p>Metrics</p>
		<hr />
	</div>
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

	.header p {
		margin-top: 0.3rem;
		margin-bottom: 0;
		padding-left: 0.3rem;
		font-size: 0.7rem;
		color: var(--foreground-3);
	}

	.header hr {
		margin: 0.2rem;
		border-color: var(--foreground-5);
	}

	button {
		display: flex;
		align-items: center;
		padding-left: 1rem;
		width: 100%;
		height: 2rem;
		border-radius: 0.5rem;
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
