/**
 * @file
 * This file contains group management code, eg adding a user to a group, removing a user from a group.
 */

import { type User, type Group, type Permission, allPermissions } from './types';
import { apiAddress } from './main';
import { addToast, ToastType } from './toast';

/**
 * From the list returned when all groups are fetched.
 */
export interface GroupListEntry {
	id: number;
	name: string;
	members: User[];
	permissions: Permission[];
}

/**
 * Add the provided user to the provided group
 * @param user The user you want to add to the group
 * @param group The group you want to add to the user
 */
export async function addUserToGroup(user: User, group: Group) {
	if (user.groups) {
		user.groups.push(group);
	}
	const r = await fetch(`${apiAddress}/api/users/groups/${user.id}`, {
		credentials: 'include',
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({
			group_ids: [group.id]
		})
	});
	if (r.ok) {
		addToast(
			`The ${group.name} group was added to the user ${user.username}`,
			ToastType.Info,
			true,
			1500
		);
	} else {
		console.error(
			`Add group to user operation failed with status code ${r.status}: ${await r.text()}`
		);
	}
}

/**
 * Remove the provided user from a group
 * @param user The user to remove from a group
 * @param group The group to remove the user from.
 */
export async function removeUserFromGroup(user: User, group: Group) {
	if (user.groups) {
		user.groups = user.groups.filter((g) => g.id !== group.id);
	}
	const r = await fetch(`${apiAddress}/api/users/groups/${user.id}`, {
		credentials: 'include',
		method: 'DELETE',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({
			group_ids: [group.id]
		})
	});
	if (r.ok) {
		addToast(
			`The ${group.name} group was removed from ${user.username}`,
			ToastType.Info,
			true,
			1500
		);
	} else {
		console.error(
			`Remove group from user operation failed with status code ${r.status}: ${await r.text()}`
		);
	}
}

/**
 * Give the provided group a new permission.
 * @param group The group to modify
 * @param permission The permission to add
 */
export async function addPermissionToGroup(group: GroupListEntry, permission: Permission) {
	group.permissions.push(permission as Permission);
	const response = await fetch(`${apiAddress}/api/groups/${group.id}/permissions`, {
		credentials: 'include',
		method: 'PUT',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({
			permissions: group.permissions
		})
	});
	if (response.ok) {
		addToast(
			`${group.name} was given the permission "${allPermissions.get(permission)}"`,
			ToastType.Info,
			true,
			1500
		);
	} else {
		console.error(
			`Add permission to group operation failed with status code ${response.status}: ${await response.text()}`
		);
	}
}

/**
 * Remove a permission from the provided group
 * @param group The group to remove the permission from
 * @param permission The permission to remove
 */
export async function removePermissionFromGroup(group: GroupListEntry, permission: Permission) {
	group.permissions = group.permissions.filter((p) => p !== permission);
	const response = await fetch(`${apiAddress}/api/groups/${group.id}/permissions`, {
		credentials: 'include',
		method: 'PUT',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({
			permissions: group.permissions
		})
	});
	if (response.ok) {
		addToast(
			`${group.name} lost the permission "${allPermissions.get(permission)}"`,
			ToastType.Info,
			true,
			1500
		);
	} else {
		console.error(
			`Remove permission from group failed with status code ${response.status}: ${await response.text()}`
		);
	}
}

/**
 * Remove all members from a group and then delete it
 * @param group The group to delete
 */
export async function deleteGroup(group: GroupListEntry) {
	await fetch(`${apiAddress}/api/groups/${group.id}`, { credentials: 'include', method: 'DELETE' });
	addToast(`The ${group.name} group was deleted.`, ToastType.Info, true, 1500);
}
