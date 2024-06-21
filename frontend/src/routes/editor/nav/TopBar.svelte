<script lang="ts">
	import { onMount } from 'svelte';

	let username: string = '';

	onMount(() => {
		const cookieJar = document.cookie.split('; ');
		for (const cookie of cookieJar) {
			const [key, value] = cookie.split('=');
			if (key === 'username') {
				username = value;
			}
		}
	});
</script>

<div class="top-bar">
	<div class="account">
		<!-- <span class="dot"></span> -->
		<!-- <svg xmlns="http://www.w3.org/2000/svg" height="2rem" viewBox="0 -960 960 960" width="2rem"><path d="M234-276q51-39 114-61.5T480-360q69 0 132 22.5T726-276q35-41 54.5-93T800-480q0-133-93.5-226.5T480-800q-133 0-226.5 93.5T160-480q0 59 19.5 111t54.5 93Zm246-164q-59 0-99.5-40.5T340-580q0-59 40.5-99.5T480-720q59 0 99.5 40.5T620-580q0 59-40.5 99.5T480-440Zm0 360q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q53 0 100-15.5t86-44.5q-39-29-86-44.5T480-280q-53 0-100 15.5T294-220q39 29 86 44.5T480-160Zm0-360q26 0 43-17t17-43q0-26-17-43t-43-17q-26 0-43 17t-17 43q0 26 17 43t43 17Zm0-60Zm0 360Z"/></svg> -->
		<!-- <p><b>{username}</b></p> -->
		<p>{username}</p>
		<svg xmlns="http://www.w3.org/2000/svg" height="2rem" viewBox="0 -960 960 960" width="2rem"
			><path
				d="m370-80-16-128q-13-5-24.5-12T307-235l-119 50L78-375l103-78q-1-7-1-13.5v-27q0-6.5 1-13.5L78-585l110-190 119 50q11-8 23-15t24-12l16-128h220l16 128q13 5 24.5 12t22.5 15l119-50 110 190-103 78q1 7 1 13.5v27q0 6.5-2 13.5l103 78-110 190-118-50q-11 8-23 15t-24 12L590-80H370Zm70-80h79l14-106q31-8 57.5-23.5T639-327l99 41 39-68-86-65q5-14 7-29.5t2-31.5q0-16-2-31.5t-7-29.5l86-65-39-68-99 42q-22-23-48.5-38.5T533-694l-13-106h-79l-14 106q-31 8-57.5 23.5T321-633l-99-41-39 68 86 64q-5 15-7 30t-2 32q0 16 2 31t7 30l-86 65 39 68 99-42q22 23 48.5 38.5T427-266l13 106Zm42-180q58 0 99-41t41-99q0-58-41-99t-99-41q-59 0-99.5 41T342-480q0 58 40.5 99t99.5 41Zm-2-140Z"
			/></svg
		>
	</div>
</div>

<style>
	/* @import '/css/theme.css'; */
	.top-bar {
		/* float: right; */
		display: flex;
		background-color: var(--background-2);
		height: 4rem;
		justify-content: right;
		align-items: center;
		width: calc(100vw - var(--sidebar-width));
	}

	.top-bar * {
		margin-right: 0.6rem;
	}

	.account {
		display: flex;
		flex-direction: row;
		align-items: center;
	}

	.account p {
		color: var(--foreground-2);
		font-family: var(--font-family);
		font-size: larger;
		margin-right: 0.4rem;
		font-weight: 350;
	}

	.account svg {
		fill: var(--foreground-2);
	}
</style>
