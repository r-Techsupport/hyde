<!-- TODO: none of these changes are synced to the database -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { tick } from 'svelte';

	const allPermissions = ['Manage Content', 'Manage Users'];
	let groups = [
		{
			id: 0,
			name: 'Admin',
			permissions: ['Manage Content', 'Manage Users']
		},
		{
			id: 1,
			name: 'Group 1',
			permissions: ['Manage Users']
		}
	];
	let selectedGroup = 0;

	let showNewGroupInput = false;
	let newGroupInput: HTMLInputElement;

	function userSelectHandler(e: MouseEvent) {
		const target = e.target as HTMLElement;
		selectedGroup = Number(target.parentElement!.id);
		for (const permission of allPermissions) {
			const element = document.getElementById(permission) as HTMLInputElement;
			if (groups[selectedGroup].permissions.includes(permission)) {
				element.checked = true;
			} else {
				element.checked = false;
			}
		}
	}

	function checkboxToggleHandler(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.checked) {
			console.log(
				`The ${target.id} permission was added to the group ${groups[selectedGroup].name}`
			);
		} else {
			console.log(
				`The ${target.id} permission was removed from the group ${groups[selectedGroup].name}`
			);
		}
	}

	onMount(() => {
		for (const permission of allPermissions) {
			const element = document.getElementById(permission) as HTMLInputElement;
			if (groups[selectedGroup].permissions.includes(permission)) {
				element.checked = true;
			} else {
				element.checked = false;
			}
		}
	});
</script>

<div class="container">
	<ul class="group-menu">
		<li class="header"><u>Groups</u></li>
		{#each groups as group}
			<li class={selectedGroup == group.id ? 'selected-group' : ''} id={group.id.toString()}>
				<button on:click={userSelectHandler}>
					<span>{group.name}</span>
				</button>
			</li>
		{/each}
		<!-- When a user is creating a new group, this is the input field -->
		{#if showNewGroupInput}
			<li>
				<input
					bind:this={newGroupInput}
					on:blur={() => {
						showNewGroupInput = false;
					}}
					on:keydown={(e) => {
						if (e.key === 'Enter') {
							groups = [
								...groups,
								{ id: groups.length, name: newGroupInput.value, permissions: [] }
							];
							showNewGroupInput = false;
						}
					}}
					class="new-group-input"
					type="text"
				/>
			</li>
		{/if}
		<!-- The "new group" button -->
		<li>
			<button
				on:click={async () => {
					showNewGroupInput = true;
					await tick();
					newGroupInput.focus();
				}}
			>
				<span>+</span>
			</button>
		</li>
	</ul>
	<ul class="permission-menu">
		<li class="header" style="justify-content: left; margin-left: 30%;"><u>Permissions</u></li>
		{#each allPermissions as group}
			<li>
				<label for={group} class="checkbox-label">
					<input on:change={checkboxToggleHandler} id={group} type="checkbox" name={group} />
					{group}
				</label>
			</li>
		{/each}
	</ul>
</div>

<style>
	.container {
		width: 100%;
		display: flex;
		fill: var(--foreground-3);
	}

	.group-menu {
		margin-top: 0;
		margin-bottom: 0;
		padding-left: 0;
		height: 100%;
		border-right: 1.5px solid var(--background-3);
		list-style-type: none;
		width: 50%;
		overflow-y: scroll;
	}

	.permission-menu {
		margin-top: 0;
		margin-bottom: 0;
		padding-left: 0;
		list-style-type: none;
		width: 50%;
		height: 100%;
		overflow-y: scroll;
	}

	.group-menu span {
		vertical-align: middle;
		pointer-events: none;
	}

	.group-menu button {
		padding: 0.7rem;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: transparent;
		color: var(--foreground-3);
		font-size: 0.95rem;
		border: none;
		width: 100%;
	}

	.group-menu button:hover {
		background-color: var(--background-3);
		cursor: pointer;
	}

	.selected-group {
		background-color: var(--background-2);
	}

	.checkbox-label {
		width: 100%;
		height: 3rem;
		display: flex;
		align-items: center;
		cursor: pointer;
	}

	.checkbox-label input {
		margin: 0.5rem;
		margin-left: 20%;
	}

	.checkbox-label:hover {
		background-color: var(--background-3);
	}

	.new-group-input {
		/* height:  */
		padding: 0.7rem;
		width: 100%;
		background-color: var(--background-3);
		border: none;
		color: var(--foreground-3);
	}

	.header {
		display: flex;
		place-content: center;
		height: 2rem;
	}
</style>
