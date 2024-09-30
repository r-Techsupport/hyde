<script lang="ts">
	import { blur } from 'svelte/transition';
	import { createEventDispatcher, onMount } from 'svelte';
	import { apiAddress } from '$lib/main';
	import { Permission, type User } from '$lib/types';

	export let visible = false;
	let showAdminDashboard = false;
	onMount(async () => {
		// This could probably be generalized into a single call that loads
		// into a svelte store or something, rather than this call being made multiple times
		const currentUser: User = await (
			await fetch(`${apiAddress}/api/users/me`, { credentials: 'include' })
		).json();
		if (currentUser.permissions.includes(Permission.ManageUsers)) {
			showAdminDashboard = true;
		}
	});
	const dispatch = createEventDispatcher();
</script>

{#if visible}
	<!-- The "Click anywhere else to hide the dialogue" functionality is implemented by having a div that sits behind the settings menu, listening for clicks -->
	<div
		on:click={() => {
			visible = false;
		}}
		on:keydown={() => {
			visible = false;
		}}
		role="none"
		class="backdrop"
	></div>
	<div transition:blur={{ duration: 75 }} class="container">
		<div class="settings-header">
			<p>Settings</p>
			<hr />
		</div>
		<!-- Admin Dashboard -->
		{#if showAdminDashboard}
			<div>
				<button
					on:click={() => {
						dispatch('showadmindashboard');
					}}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						height="1.5rem"
						viewBox="0 -960 960 960"
						width="1.5rem"
						fill="#e8eaed"
						><path
							d="M756-120 537-339l84-84 219 219-84 84Zm-552 0-84-84 276-276-68-68-28 28-51-51v82l-28 28-121-121 28-28h82l-50-50 142-142q20-20 43-29t47-9q24 0 47 9t43 29l-92 92 50 50-28 28 68 68 90-90q-4-11-6.5-23t-2.5-24q0-59 40.5-99.5T701-841q15 0 28.5 3t27.5 9l-99 99 72 72 99-99q7 14 9.5 27.5T841-701q0 59-40.5 99.5T701-561q-12 0-24-2t-23-7L204-120Z"
						/></svg
					>
					Admin Dashboard
				</button>
			</div>
		{/if}
		<!-- Logout -->
		<div>
			<button
				on:click={async () => {
					document.cookie =
						'username=; expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=None; Secure';
					await fetch(`${apiAddress}/api/logout`);
					location.reload();
				}}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					height="1.5rem"
					viewBox="0 -960 960 960"
					width="1.5rem"
					fill="#e8eaed"
					><path
						d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h280v80H200v560h280v80H200Zm440-160-55-58 102-102H360v-80h327L585-622l55-58 200 200-200 200Z"
					/></svg
				>
				Logout
			</button>
		</div>
		<!-- TODO: theme toggle -->
	</div>
{/if}

<style>
	:root {
		font-family: var(--font-family);
	}

	.container {
		background-color: var(--background-3);
		position: absolute;
		display: flex;
		flex-direction: column;
		padding: 0.3rem;
		top: 4rem;
		right: 0;
		width: 15rem;
	}

	button * {
		margin-right: 0.2rem;
		vertical-align: middle;
	}

	.container * {
		margin-left: 0.1rem;
		margin-right: 0.1rem;
		font-size: 1rem;
		color: var(--foreground-1);
	}

	.container div {
		width: 100%;
	}

	.backdrop {
		position: absolute;
		top: 0;
		right: 0;
		width: 100vw;
		height: 100vh;
	}

	.settings-header p {
		margin-top: 0.3rem;
		margin-bottom: 0;
		padding-left: 0.3rem;
		font-size: 0.7rem;
		color: var(--foreground-3);
	}

	.settings-header hr {
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
		fill: var(--foreground-1);
		padding-right: 0.2rem;
	}
</style>
