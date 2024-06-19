<!-- The container where rendered toasts are displayed -->
<script lang="ts">
	import Toast from './Toast.svelte';

	import { dismissToast, toasts } from '$lib/toast';
</script>

{#if $toasts}
	<div>
		{#each $toasts as toast (toast.id)}
			<Toast
				type={toast.type}
				dismissible={toast.dismissible}
				on:dismiss={() => dismissToast(toast.id ?? -1)}>{toast.message}</Toast
			>
		{/each}
	</div>
{/if}

<style>
	div {
		position: absolute;
		top: 0;
		right: 0;
		display: flex;
		margin-top: 1rem;
		margin-right: 0.5rem;
		flex-direction: column;
		/* z-index: 1000; */
	}
</style>
