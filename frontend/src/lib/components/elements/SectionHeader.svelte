<script lang="ts">
	interface Props {
		children?: import('svelte').Snippet;
	}

	let { children }: Props = $props();
</script>

<!--
 A label with a horizontal line beneath it, used to distinguish different sections of a menu.
   - The `--scale` CSS variable can be used to adjust the size of the header
   - The `--text-color` CSS variable can be used to adjust the color of the header text.
-->
<div class="header">
	<!-- TODO: Figure out a more idiomatic way to represent this, eg `h1` -->
	<p>{@render children?.()}</p>
	<hr />
</div>

<style>
	.header p {
		margin-top: calc(var(--scale, 0.7rem) / 2.25);
		margin-bottom: 0;
		padding-left: calc(var(--scale, 0.7rem) / 2.25);
		font-size: var(--scale, 0.7rem);
		color: var(--text-color, --foreground-3);
	}

	.header hr {
		/* margin: calc(--scale / 3); */
		margin: 0.2rem;

		/* border-color: var(--break-color, --background-1); */

		/* border-: var(--foreground-5); */

		/* border: 1px solid var(--break-color, --foreground-5); */
		border: none;
		border-top: 1px solid var(--foreground-5);
	}
</style>
