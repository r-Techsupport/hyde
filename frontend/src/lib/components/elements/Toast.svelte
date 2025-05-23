<!-- A single toast notification -->
<!-- Adapted from https://svelte.dev/repl/0091c8b604b74ed88bb7b6d174504f50?version=3.35.0 -->
<script lang="ts">
	import { ToastType } from '$lib/toast';
	import { createEventDispatcher } from 'svelte';
	import { fade } from 'svelte/transition';
	import { quartOut } from 'svelte/easing';
	//   import SuccessIcon from "./SuccessIcon.svelte";
	//   import ErrorIcon from "./ErrorIcon.svelte";
	//   import InfoIcon from "./InfoIcon.svelte";
	//   import CloseIcon from "./CloseIcon.svelte";

	const dispatch = createEventDispatcher();

	interface Props {
		type?: string;
		dismissible?: boolean;
		children?: import('svelte').Snippet;
	}

	let { type = 'error', dismissible = true, children }: Props = $props();
</script>

<article class={type} role="alert" transition:fade={{ duration: 150, easing: quartOut }}>
	{#if type === ToastType.Success}
		<svg
			width="2rem"
			style="text-align: center; display: inline-block;"
			aria-hidden="true"
			focusable="false"
			role="img"
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 512 512"
		>
			<path
				fill="currentColor"
				d="M256 8C119.033 8 8 119.033 8 256s111.033 248 248 248 248-111.033 248-248S392.967 8 256 8zm0 464c-118.664 0-216-96.055-216-216 0-118.663 96.055-216 216-216 118.664 0 216 96.055 216 216 0 118.663-96.055 216-216 216zm141.63-274.961L217.15 376.071c-4.705 4.667-12.303 4.637-16.97-.068l-85.878-86.572c-4.667-4.705-4.637-12.303.068-16.97l8.52-8.451c4.705-4.667 12.303-4.637 16.97.068l68.976 69.533 163.441-162.13c4.705-4.667 12.303-4.637 16.97.068l8.451 8.52c4.668 4.705 4.637 12.303-.068 16.97z"
			/>
		</svg>
	{:else if type === ToastType.Error}
		<svg
			width="1rem"
			style="text-align: center; display: inline-block;"
			aria-hidden="true"
			focusable="false"
			role="img"
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 352 512"
		>
			<path
				fill="currentColor"
				d="M242.72 256l100.07-100.07c12.28-12.28 12.28-32.19 0-44.48l-22.24-22.24c-12.28-12.28-32.19-12.28-44.48 0L176 189.28 75.93 89.21c-12.28-12.28-32.19-12.28-44.48 0L9.21 111.45c-12.28 12.28-12.28 32.19 0 44.48L109.28 256 9.21 356.07c-12.28 12.28-12.28 32.19 0 44.48l22.24 22.24c12.28 12.28 32.2 12.28 44.48 0L176 322.72l100.07 100.07c12.28 12.28 32.2 12.28 44.48 0l22.24-22.24c12.28-12.28 12.28-32.19 0-44.48L242.72 256z"
			/>
		</svg>
	{:else if type === ToastType.Warning}
		<svg
			width="2rem"
			style="text-align: center; display: inline-block;"
			aria-hidden="true"
			focusable="false"
			role="img"
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 512 512"
		>
			<path
				fill="currentColor"
				d="M256 8C119.033 8 8 119.033 8 256s111.033 248 248 248 248-111.033 248-248S392.967 8 256 8zm0 464c-118.664 0-216-96.055-216-216 0-118.663 96.055-216 216-216 118.664 0 216 96.055 216 216 0 118.663-96.055 216-216 216zm0-388c-13.255 0-24 10.745-24 24v152c0 13.255 10.745 24 24 24s24-10.745 24-24V104c0-13.255-10.745-24-24-24zm0 280c-18.625 0-36.708 9.305-45.968 25.525-3.226 5.708-1.288 12.742 4.42 15.969 5.711 3.226 12.742 1.288 15.969-4.42C239.298 396.138 246 379.644 246 360c0-13.008 10.989-24 24-24s24 10.992 24 24c0 27.13-12.542 53.915-33.006 70.495-5.683 4.733-6.647 12.717-2.666 18.4 4.026 5.669 11.719 7.153 17.56 3.306C257.835 442.928 272 406.267 272 360c0-36.129-29.871-66-66-66z"
			/>
		</svg>
	{:else}
		<svg
			width="1rem"
			style="text-align: center; display: inline-block;"
			aria-hidden="true"
			focusable="false"
			role="img"
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 512 512"
		>
			<path
				fill="currentColor"
				d="M256 40c118.621 0 216 96.075 216 216 0 119.291-96.61 216-216 216-119.244 0-216-96.562-216-216 0-119.203 96.602-216 216-216m0-32C119.043 8 8 119.083 8 256c0 136.997 111.043 248 248 248s248-111.003 248-248C504 119.083 392.957 8 256 8zm-36 344h12V232h-12c-6.627 0-12-5.373-12-12v-8c0-6.627 5.373-12 12-12h48c6.627 0 12 5.373 12 12v140h12c6.627 0 12 5.373 12 12v8c0 6.627-5.373 12-12 12h-72c-6.627 0-12-5.373-12-12v-8c0-6.627 5.373-12 12-12zm36-240c-17.673 0-32 14.327-32 32s14.327 32 32 32 32-14.327 32-32-14.327-32-32-32z"
			/>
		</svg>
	{/if}

	<div class="text">
		{@render children?.()}
	</div>

	{#if dismissible}
		<button class="close" aria-label="Close" onclick={() => dispatch('dismiss')}>
			<svg
				width="1rem"
				style="text-align: center; display: inline-block;"
				aria-hidden="true"
				focusable="false"
				role="img"
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 352 512"
			>
				<path
					fill="currentColor"
					d="M242.72 256l100.07-100.07c12.28-12.28 12.28-32.19 0-44.48l-22.24-22.24c-12.28-12.28-32.19-12.28-44.48 0L176 189.28 75.93 89.21c-12.28-12.28-32.19-12.28-44.48 0L9.21 111.45c-12.28 12.28-12.28 32.19 0 44.48L109.28 256 9.21 356.07c-12.28 12.28-12.28 32.19 0 44.48l22.24 22.24c12.28 12.28 32.2 12.28 44.48 0L176 322.72l100.07 100.07c12.28 12.28 32.2 12.28 44.48 0l22.24-22.24c12.28-12.28 12.28-32.19 0-44.48L242.72 256z"
				/>
			</svg>
		</button>
	{/if}
</article>

<style>
	article {
		color: var(--foreground-0);
		font-family: var(--font-family);
		padding: 0.75rem 1.5rem;
		border-radius: 0.2rem;
		display: flex;
		align-items: center;
		margin: 0 0.5rem 0.5rem auto;
		width: 20rem;
		float: right;
		white-space: pre-line;
	}

	.error {
		background: var(--toast-error);
	}

	.success {
		background: var(--toast-success);
	}

	.info {
		background: var(--toast-info);
	}

	.warning {
		background: var(--toast-warning);
	}

	.text {
		margin-left: 1rem;
		margin-right: 0.5rem;
	}

	.close {
		cursor: pointer;
	}

	button {
		color: white;
		background: transparent;
		border: 0 none;
		padding: 0;
		margin: 0 0 0 auto;
		line-height: 1;
		font-size: 1rem;
	}
</style>
