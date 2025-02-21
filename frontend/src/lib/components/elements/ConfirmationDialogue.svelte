<!-- A generic confirmation confirm/cancel dialogue box -->
<script lang="ts">
	interface Props {
		/**
		 * If defined, the code that runs when a user hits "Confirm".
		 */
		confirmHandler?: () => void;
		/**
		 * If defined, the code that runs when the user hits "Deny" or
		 * clicks out of the widget.
		 */
		cancelHandler?: () => void;
		confirmText?: string;
		cancelText?: string;
		visible: boolean;
		children?: import('svelte').Snippet;
	}

	let {
		confirmHandler = () => {},
		cancelHandler = () => {},
		confirmText = 'Confirm',
		cancelText = 'Cancel',
		visible = $bindable(),
		children
	}: Props = $props();
</script>

<div class="backdrop"></div>
<div class="container">
	<div class="inserted-content">
		{@render children?.()}
	</div>
	<div class="control-buttons">
		<button
			onclick={() => {
				visible = false;
				cancelHandler();
			}}
		>
			{cancelText}
		</button>
		<button
			onclick={() => {
				visible = false;
				confirmHandler();
			}}
		>
			{confirmText}
		</button>
	</div>
</div>

<style>
	.backdrop {
		position: absolute;
		width: 100vw;
		height: 100vh;
		left: 0;
		top: 0;
		background-color: var(--background-0);
		opacity: 0.4;
	}

	.container {
		position: absolute;
		z-index: 1;
		background-color: var(--background-2);
		border-radius: 2px;
		top: 40%;
		left: 50%;
		transform: translate(-50%, -50%);
	}

	.inserted-content {
		margin: 0.5rem;
	}

	.control-buttons {
		display: flex;
		justify-content: right;
		margin: 0.3rem;
	}

	.control-buttons button {
		cursor: pointer;
		margin: 0.1rem;
		font-size: 0.9rem;
		background-color: transparent;
		color: var(--foreground-0);
		border: 1px solid var(--foreground-0);
		border-radius: 2px;
	}

	.control-buttons button:hover {
		color: var(--background-2);
		background-color: var(--foreground-1);
	}
</style>
