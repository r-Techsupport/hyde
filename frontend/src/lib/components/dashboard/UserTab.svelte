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

	let searchQuery = $state('');
	let displayedUsers: User[] = $derived(users);
	function searchUserHandler(e: Event) {
		const target = e.target as HTMLInputElement;
		searchQuery = target.value.toLowerCase().trim();
		if (searchQuery) {
			displayedUsers = users.filter((user) => user.username.includes(searchQuery));
		} else {
			displayedUsers = users;
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
		<input class="search" type="text" placeholder="Search users" oninput={searchUserHandler} />
		{#each users.entries() as [index, user] (index)}
			{#if displayedUsers.includes(user)}
				<li class={selectedUser == index ? 'selected-user' : ''} id={index.toString()}>
					<button onclick={userSelectHandler}>
						<img src={user.avatar_url} alt="User avatar" width="25rem" height="25rem" />
						<span>{user.username}</span>
					</button>
				</li>
			{/if}
		{/each}
		{#if displayedUsers.length == 0}
			<span>No users found.</span>
		{/if}
	</ul>
	<ul class="group-menu">
		<SectionHeader>Groups</SectionHeader>
		{#each allGroups as group (group.name)}
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
		padding: 0 0.2rem 0 0.2rem;
		height: 100%;
		border-right: 1.5px solid var(--background-3);
		list-style-type: none;
		width: 50%;
		overflow-y: auto;
	}

	.group-menu {
		margin-top: 0;
		margin-bottom: 0;
		padding: 0 0.2rem 0 0.2rem;
		list-style-type: none;
		width: 50%;
		height: 100%;
		overflow-y: scroll;
	}

	.search {
		padding: 0;
		margin-bottom: 0.4rem;
		color: var(--foreground-3);
		background-color: var(--background-3);
		border: none;
		width: 100%;
		height: 1.75rem;
		border-radius: 0.25rem;
		text-indent: 1rem;
	}

	.search:focus {
		outline: none;
		border-bottom: 1px solid var(--foreground-3);
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
