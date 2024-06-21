<script lang="ts">
	import { onMount } from 'svelte';
	import { apiAddress } from '$lib/net';

	let redirectUrl = '';

	function loginHandler() {
		window.location.href = redirectUrl;
	}

	onMount(async () => {
		redirectUrl = await (await fetch(`${apiAddress}/api/oauth/url`)).text();
	});
</script>

<div class="login-container">
	<h2>Login to Hyde</h2>
	<button on:click={loginHandler}>
		<img
			src="/assets/discord-logo.svg"
			alt="discord-logo"
			class="discord-logo"
			width="35rem"
			height="35rem"
		/>
		<span> Click here to authenticate with Discord </span>
	</button>
</div>

<style>
	:root {
		background-color: var(--background-0);
	}

	.login-container {
		width: 30rem;
		height: 30rem;
		border-radius: 0.2rem;
		border: 1px solid var(--foreground-5);
		margin: auto;
		margin-top: 8rem;
		display: flex;
		flex-direction: column;
		color: var(--foreground-0);
		font-family: var(--font-family);
	}

	.login-container h2 {
		margin-top: 2rem;
		text-align: center;
	}

	button {
		background-color: #313338;
		color: #f2f3f5;
		margin: 0 auto;
		height: 4rem;
		font-family: var(--font-family);
		font-size: large;
		font-weight: 500;
		border-radius: 0.1rem;
		border: 1px solid var(--foreground-5);
		padding: 1rem;
		cursor: pointer;
	}

	button * {
		vertical-align: middle;
	}

	.discord-logo {
		margin-right: 0.5rem;
	}
</style>
