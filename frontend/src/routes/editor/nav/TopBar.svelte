<script lang="ts">
</script>

<div class="top-bar">
	<span class="dot"></span>
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
		margin-right: 0.5rem;
	}

	.dot {
		display: flex;
		height: 3rem;
		width: 3rem;
		background-color: var(--foreground-5);
		border-radius: 50%;
		border: 1px var(--background-1) solid;
		float: right;
	}
</style>
