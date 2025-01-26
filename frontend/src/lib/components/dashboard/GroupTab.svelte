<!-- TODO: none of these changes are synced to the database -->
<script lang="ts">
	import { apiAddress } from '$lib/main';
	import { addToast, ToastType } from '$lib/toast';
	import { allPermissions, Permission } from '$lib/types';
	import { onMount } from 'svelte';
	import { tick } from 'svelte';
	import { addPermissionToGroup, deleteGroup, removePermissionFromGroup } from '$lib/groups';
	import { type GroupListEntry } from '$lib/groups';
	import SectionHeader from '../elements/SectionHeader.svelte';

	let groups: GroupListEntry[] = $state([]);
	let selectedGroup = $state(1);

	let showNewGroupInput = $state(false);
	let newGroupInput: HTMLInputElement = $state();

	function userSelectHandler(e: MouseEvent) {
		const target = e.target as HTMLElement;
		selectedGroup = Number(target.parentElement!.id);
		for (const permission of allPermissions.keys()) {
			const element = document.getElementById(permission) as HTMLInputElement;
			if (groups[selectedGroup].permissions.includes(permission)) {
				element.checked = true;
			} else {
				element.checked = false;
			}
		}
	}

	async function checkboxToggleHandler(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.checked) {
			addPermissionToGroup(groups[selectedGroup], target.id as Permission);
		} else {
			removePermissionFromGroup(groups[selectedGroup], target.id as Permission);
		}
	}

	onMount(async () => {
		groups = await (await fetch(`${apiAddress}/api/groups`, { credentials: 'include' })).json();
		for (const permission of allPermissions.keys()) {
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
		<SectionHeader>Groups</SectionHeader>
		{#each groups.entries() as [index, group]}
			<!-- Prevent people from modifying the permissions on the admin group -->
			{#if group.name !== 'Admin'}
				<li class={selectedGroup == index ? 'selected-group' : ''} id={index.toString()}>
					<button onclick={userSelectHandler}>
						<!-- TODO: trashcan on right, label on center -->
						<span>{group.name}</span>
						<svg
							onclick={async () => {
								await deleteGroup(group);
								groups = groups.filter((g) => g.name !== group.name);
							}}
							onkeydown={async () => {
								await deleteGroup(group);
								groups = groups.filter((g) => g.name !== group.name);
							}}
							role="button"
							tabindex="0"
							xmlns="http://www.w3.org/2000/svg"
							height="24px"
							viewBox="0 -960 960 960"
							width="24px"
							><path
								d="M280-120q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520ZM360-280h80v-360h-80v360Zm160 0h80v-360h-80v360ZM280-720v520-520Z"
							/></svg
						>
					</button>
				</li>
			{/if}
		{/each}
		<!-- When a user is creating a new group, this is the input field -->
		{#if showNewGroupInput}
			<li>
				<input
					bind:this={newGroupInput}
					onblur={() => {
						showNewGroupInput = false;
					}}
					onkeydown={async (e) => {
						if (e.key === 'Enter') {
							// TODO: migrate to a createGroup function
							const newGroup = await (
								await fetch(`${apiAddress}/api/groups`, {
									method: 'POST',
									credentials: 'include',
									headers: { 'Content-Type': 'application/json' },
									body: JSON.stringify({ group_name: newGroupInput.value })
								})
							).json();
							groups = [...groups, newGroup];
							addToast(
								`The ${newGroup.name} group was created successfully`,
								ToastType.Info,
								true,
								1500
							);
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
				onclick={async () => {
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
		<SectionHeader>Permissions</SectionHeader>
		{#each allPermissions as [permission, label]}
			<li>
				<label for={permission} class="checkbox-label">
					<input
						onchange={checkboxToggleHandler}
						id={permission}
						type="checkbox"
						name={permission}
					/>
					{label}
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

	.group-menu svg {
		margin-right: 0.3rem;
		fill: transparent;
	}

	.group-menu button:hover svg {
		fill: var(--background-4);
	}

	.group-menu button:hover svg:hover {
		background-color: var(--background-3);
		fill: var(--red);
		border-radius: 0.2rem;
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
</style>
