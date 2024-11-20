<script lang="ts">
	interface Props {
		visible?: boolean;
	}

	let { visible = $bindable(false) }: Props = $props();
</script>

{#if visible}
	<div
		class="background-cover"
		onclick={() => {
			visible = false;
		}}
		tabindex="0"
		onkeypress={() => {
			visible = false;
		}}
		role="button"
	></div>
	<div class="change-dialogue">
		<svg
			onclick={() => {
				visible = false;
			}}
			onkeypress={() => {
				visible = false;
			}}
			role="button"
			tabindex="0"
			xmlns="http://www.w3.org/2000/svg"
			height="24px"
			viewBox="0 -960 960 960"
			width="24px"
			fill="#e8eaed"
		>
			<path
				d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"
			/>
		</svg>
		<p>
			You have unsaved changes, please commit or revert changes to the current file before
			switching.
		</p>
	</div>
{/if}

<style>
	.background-cover {
		position: absolute;
		background-color: var(--background-0);
		opacity: 0.9;
		width: 100vw;
		height: 100vh;
	}

	.change-dialogue {
		position: absolute;

		/* Centering in the window */
		left: calc((100vw + var(--sidebar-width)) / 2);
		top: 40%;
		transform: translate(-50%, -50%);

		/* Shape */
		width: 20rem;

		/* Appearance */
		border: 1px solid var(--background-2);
		border-radius: 5px;
		background-color: var(--background-1);
		color: var(--foreground-0);
		font-family: var(--font-family);
	}

	.change-dialogue p {
		padding-left: 2rem;
		padding-right: 2rem;
	}

	.change-dialogue svg {
		cursor: pointer;
		margin-top: 0.2rem;
		margin-right: 0.2rem;
		float: right;
	}
</style>
