// TODO: rename this file to remove the .d, it's bad

export enum Permission {
	ManageUsers = 'ManageUsers',
	ManageContent = 'ManageContent'
}

export const allPermissions: Map<Permission, string> = new Map();
allPermissions.set(Permission.ManageContent, 'Manage Content');
allPermissions.set(Permission.ManageUsers, 'Manage Users');

export interface User {
	id: number;
	username: string;
	avatar_url: string;
	groups?: Group[];
	permissions: Permission[];
}

export interface Group {
	id: number;
	name: string;
}
