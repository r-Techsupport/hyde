<script lang="ts">
	export let visible: boolean = true;
</script>
{#if visible}
	<div
		class="background-cover"
	></div>
	<div class="loading-icon">
	</div>
{/if}

<style>
	.background-cover {
		position: absolute;
		background-color: var(--background-0);
		opacity: 90%;
		width: 100vw;
		height: 100vh;
	}

	/* .change-dialogue {
		position: absolute;
		left: calc((100vw + var(--sidebar-width)) / 2);
		top: 40%;
		transform: translate(-50%, -50%);
		width: 20rem;
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
	} */

    .loading-icon {
        position: absolute;
        left: 50%;
        top: 40%;
        width: 4rem;
        height: 4rem;
        /* margin: -4rem 0 0 8rem; */
        border: 0.5rem solid rgba(0, 51, 255, 0);
        border-radius: 50%;
        border-top: 0.5rem solid #ff0000;
        -webkit-animation: spin 1.0s linear infinite;
        animation: spin 1.0s linear infinite;
    }


    @-webkit-keyframes spin {
        0% { -webkit-transform: rotate(0deg); }
        100% { -webkit-transform: rotate(360deg); }
    }

    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }
</style>