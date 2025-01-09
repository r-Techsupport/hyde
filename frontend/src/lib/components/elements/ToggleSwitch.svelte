<!-- https://svelte.dev/playground/d65a4e9f0ae74d1eb1b08d13e428af32?version=3.35.0 -->

<script lang="ts">
	export let size: number = 1;
	export let checked = true;
	export let onToggle: (checked: boolean) => undefined = () => {};

	const uniqueID = Math.floor(Math.random() * 100);

	function handleClick(event: MouseEvent) {
		const target: HTMLInputElement = event.target! as HTMLInputElement;
		const state = target.getAttribute('aria-checked');
		checked = state !== 'true';
		onToggle(checked);
	}
</script>

<div class="toggle-switch" style="--size={size}">
	<span><slot /></span>
	<button
		role="switch"
		aria-checked={checked}
		aria-labelledby={`switch-${uniqueID}`}
		on:click={handleClick}
		style="--size={size}"
	>
	</button>
</div>

<style>
	.toggle-switch {
		display: flex;
		box-sizing: border-box;
		align-items: center;
		width: 100%;
	}

	.toggle-switch span {
		margin-left: calc(var(--size) / 4);
		font-size: var(--size);
		text-overflow: ellipsis;
		overflow: hidden;
		max-width: calc(100% - var(--size) * 2.3);
	}

	.toggle-switch button {
		flex-shrink: 0;
		width: calc(var(--size) * 2.3);
		height: calc(var(--size) * 1.1);
		background: var(--background-3);

		/* margin: 0 0 0 calc(var(--size)); */
		position: relative;
		border: none;
		cursor: pointer;
		margin-left: auto;
		border-radius: var(--size);
	}

	.toggle-switch button::before {
		content: '';
		position: absolute;
		width: var(--size);
		height: var(--size);
		background: var(--foreground-3);

		/* top: 0.15em; */

		/* right: 0.5em; */
		top: calc(var(--size) / 20);
		right: calc(var(--size) * 1.2);
		transition: transform 0.2s;
		border-radius: 100%;
	}

	.toggle-switch button[aria-checked='true'] {
		background-color: var(--accent-0);
	}

	.toggle-switch button[aria-checked='true']::before {
		transform: translateX(calc(var(--size) / 0.9));
		transition: transform 0.2s;
		background-color: var(--foreground-1);
	}
</style>
