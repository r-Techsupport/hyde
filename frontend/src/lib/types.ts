/**
 * @file
 * Misc type declarations
 */

/**
 * All configurable permissions
 */
export enum Permission {
	ManageUsers = 'ManageUsers',
	ManageContent = 'ManageContent'
}

/**
 * A map between the internal permission representations and the "pretty print" representation
 */
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

export interface INode {
	name: string;
	children: INode[];
}
