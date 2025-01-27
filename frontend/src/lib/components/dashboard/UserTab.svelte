<script lang="ts">
	import { addUserToGroup, removeUserFromGroup } from '$lib/groups';
	import { apiAddress } from '$lib/main';
	import { addToast, ToastType } from '$lib/toast';
	import type { Group, User } from '$lib/types';
	import { onMount } from 'svelte';
	import SectionHeader from '../elements/SectionHeader.svelte';

	// const allGroups = [{'Admin', 'Group 1', 'Group 2', 'Group 3'];
	let allGroups: Group[] = $state([
		{
			id: 1,
			name: 'Admin'
		},
		{
			id: 2,
			name: 'Mock group 2'
		}
	]);
	let users: User[] = $state([]);
	let selectedUser = $state(0);
	function userSelectHandler(e: MouseEvent) {
		const target = e.target as HTMLElement;
		selectedUser = Number(target.parentElement!.id);
		for (const group of allGroups) {
			const element = document.getElementById(group.name) as HTMLInputElement;
			if (users[selectedUser].groups!.some((g) => g.name === group.name)) {
				element.checked = true;
			} else {
				element.checked = false;
			}
		}
	}

	async function checkboxToggleHandler(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.checked) {
			const groupToAdd = allGroups.find((g) => g.name === target.id);
			if (groupToAdd) {
				await addUserToGroup(users[selectedUser], groupToAdd);
			} else {
				addToast(
					"Wasn't able to add the selected group to that user because that checkbox has an ID tied to a group that does not exist, \
						please report this to the developer.",
					ToastType.Error
				);
			}
		} else {
			const groupToRemove = allGroups.find((g) => g.name === target.id);
			if (groupToRemove) {
				await removeUserFromGroup(users[selectedUser], groupToRemove);
			}
		}
	}

	onMount(async () => {
		allGroups = await (await fetch(`${apiAddress}/api/groups`, { credentials: 'include' })).json();
		users = await (await fetch(`${apiAddress}/api/users`, { credentials: 'include' })).json();
		for (const group of allGroups) {
			const element = document.getElementById(group.name) as HTMLInputElement;
			if (users[selectedUser].groups!.some((g) => g.name === group.name)) {
				element.checked = true;
			} else {
				element.checked = false;
			}
		}
	});
</script>

<div class="container">
	<ul class="user-menu">
		<SectionHeader>Users</SectionHeader>
		{#each users.entries() as [index, user]}
			<li class={selectedUser == index ? 'selected-user' : ''} id={index.toString()}>
				<button onclick={userSelectHandler}>
					<!-- <svg
						xmlns="http://www.w3.org/2000/svg"
						height="24px"
						viewBox="0 -960 960 960"
						width="24px"
					>
						<path
							d="M234-276q51-39 114-61.5T480-360q69 0 132 22.5T726-276q35-41 54.5-93T800-480q0-133-93.5-226.5T480-800q-133 0-226.5 93.5T160-480q0 59 19.5 111t54.5 93Zm246-164q-59 0-99.5-40.5T340-580q0-59 40.5-99.5T480-720q59 0 99.5 40.5T620-580q0 59-40.5 99.5T480-440Zm0 360q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q53 0 100-15.5t86-44.5q-39-29-86-44.5T480-280q-53 0-100 15.5T294-220q39 29 86 44.5T480-160Zm0-360q26 0 43-17t17-43q0-26-17-43t-43-17q-26 0-43 17t-17 43q0 26 17 43t43 17Zm0-60Zm0 360Z"
						/>
					</svg> -->
					<img src={user.avatar_url} alt="User avatar" width="25rem" height="25rem" />
					<span>{user.username}</span>
				</button>
			</li>
		{/each}
	</ul>
	<ul class="group-menu">
		<SectionHeader>Groups</SectionHeader>
		{#each allGroups as group}
			<li>
				<label for={group.name} class="checkbox-label">
					<input
						onchange={checkboxToggleHandler}
						id={group.name}
						type="checkbox"
						name={group.name}
					/>
					{group.name}
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

	.user-menu {
		margin-top: 0;
		margin-bottom: 0;
		padding-left: 0;
		height: 100%;
		border-right: 1.5px solid var(--background-3);
		list-style-type: none;
		width: 50%;
		overflow-y: scroll;
	}

	.group-menu {
		margin-top: 0;
		margin-bottom: 0;
		padding-left: 0;
		list-style-type: none;
		width: 50%;
		height: 100%;
		overflow-y: scroll;
	}

	.user-menu img {
		margin-right: 0.4rem;
		border-radius: 50%;
		pointer-events: none;
	}

	.user-menu span {
		vertical-align: middle;
		pointer-events: none;
	}

	.user-menu button {
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

	.user-menu button:hover {
		background-color: var(--background-3);
		cursor: pointer;
	}

	.selected-user {
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
		margin-left: 30%;
	}

	.checkbox-label:hover {
		background-color: var(--background-3);
	}
</style>
