<script lang="ts">
	import GroupTab from './GroupTab.svelte';
	import UserTab from './UserTab.svelte';
	import ServerTab from './ServerTab.svelte';
	// import BranchTab from './BranchTab.svelte';
	// import BaseBranch from './BaseBranch.svelte';

	interface Props {
		dialog: HTMLDialogElement;
	}

	let { dialog = $bindable() }: Props = $props();
	let selectedTab = $state(0);
	let tabs = [
		{ name: 'User Management', id: 0, component: UserTab },
		{ name: 'Group Management', id: 1, component: GroupTab },
		{ name: 'Server Management', id: 2, component: ServerTab }
		// { name: 'Branch Management', id: 3, component: BranchTab },
		// { name: 'Base Branch', id: 4, component: BaseBranch }
	];

	// E must be defined as any because for some reason typescript thinks parentElement doesn't exist on e.target
	function tabSelectHandler(e: MouseEvent) {
		const target = e.target as HTMLElement;
		selectedTab = Number(target.parentElement!.id);
	}

	const SvelteComponent = $derived(tabs[selectedTab].component);
</script>

<dialog bind:this={dialog} class="container">
	<ul class="tab-menu">
		<li>
			<svg
				onclick={() => {
					dialog.close();
				}}
				xmlns="http://www.w3.org/2000/svg"
				height="1.5rem"
				viewBox="0 -960 960 960"
				width="1.5rem"
				onkeydown={() => {
					dialog.close();
				}}
				role="none"
			>
				<title>Exit</title>
				<path
					d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"
				/></svg
			>
		</li>
		{#each tabs as tab}
			<li class={selectedTab == tab.id ? 'selected-tab' : ''} id={tab.id.toString()}>
				<button onclick={tabSelectHandler}>{tab.name}</button>
			</li>
		{/each}
	</ul>
	<SvelteComponent />
</dialog>

<style>
	.container[open] {
		position: absolute;
		top: 10%;
		width: 40%;
		height: 60%;
		background-color: var(--background-1);
		color: var(--foreground-3);
		font-family: var(--font-family);
		border-width: 0;
		border-radius: 0.3rem;
		padding: 0;
		display: flex;
	}

	.container[open]::backdrop {
		background-color: rgb(0 0 0 / 61%);
		backdrop-filter: blur(5px);
	}

	.tab-menu {
		margin-top: 0;
		margin-bottom: 0;
		padding-left: 0;
		height: 100%;
		min-width: 8rem;
		max-width: 8rem;
		border-right: 1.5px solid var(--background-3);
		list-style-type: none;
	}

	.selected-tab {
		background-color: var(--background-2);
	}

	.tab-menu svg {
		cursor: pointer;
		margin: 0.5rem;
		fill: var(--foreground-3);
	}

	.tab-menu li {
		border-top-left-radius: 0.3rem;
		border-bottom-left-radius: 0.3rem;
	}

	.tab-menu button {
		padding: 0.7rem;
		background-color: transparent;
		color: var(--foreground-3);
		font-size: 0.95rem;
		border: none;
		width: 100%;
	}

	.tab-menu button:hover {
		background-color: var(--background-3);
		cursor: pointer;
	}
</style>
