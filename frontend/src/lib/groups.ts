import { type User, type Group, type Permission, allPermissions } from './types.d';
import { apiAddress } from './net';
import { addToast, ToastType } from './toast';

export interface GroupListEntry {
	id: number;
	name: string;
	members: User[];
	permissions: Permission[];
}

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
		addToast({
			message: `The ${group.name} group was added to the user ${user.username}`,
			type: ToastType.Info,
			dismissible: true,
			timeout: 1500
		});
	} else {
		console.error(
			`Add group to user operation failed with status code ${r.status}: ${await r.text()}`
		);
	}
}

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
		addToast({
			message: `The ${group.name} group was removed from ${user.username}`,
			type: ToastType.Info,
			dismissible: true,
			timeout: 1500
		});
	} else {
		console.error(
			`Remove group from user operation failed with status code ${r.status}: ${await r.text()}`
		);
	}
}

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
		addToast({
			message: `${group.name} was given the permission "${allPermissions.get(permission)}"`,
			type: ToastType.Info,
			dismissible: true,
			timeout: 1500
		});
	} else {
		console.error(
			`Add permission to group operation failed with status code ${response.status}: ${await response.text()}`
		);
	}
}

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
		addToast({
			message: `${group.name} lost the permission "${allPermissions.get(permission)}"`,
			type: ToastType.Info,
			dismissible: true,
			timeout: 1500
		});
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
	const usersToRemove = structuredClone(group.members);
	for (const user of usersToRemove) {
		console.log(`Removing ${group.name} in preparation for group deletion from `, user);
		await removeUserFromGroup(user, group);
	}
	await fetch(`${apiAddress}/api/groups/${group.id}`, { credentials: 'include', method: 'DELETE' });
	addToast({
		message: `The ${group.name} group was deleted.`,
		type: ToastType.Info,
		dismissible: true,
		timeout: 1500
	});
}
