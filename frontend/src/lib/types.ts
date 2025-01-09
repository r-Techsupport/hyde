/**
 * @file
 * Misc type declarations
 */

/**
 * All configurable permissions
 */
export enum Permission {
	ManageUsers = 'ManageUsers',
	ManageContent = 'ManageContent',
	ManageBranches = 'ManageBranches'
}

/**
 * A map between the internal permission representations and the "pretty print" representation
 */
export const allPermissions: Map<Permission, string> = new Map();
allPermissions.set(Permission.ManageContent, 'Manage Content');
allPermissions.set(Permission.ManageUsers, 'Manage Users');
allPermissions.set(Permission.ManageBranches, 'Manage Branches');

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

export interface Branch {
	name: string;
	isProtected: boolean;
}

export interface Issue {
	id: number;
	number: number;
	title: string;
	state: string;
	labels: string[];
	body: string;
	pull_request?: { url: string };
	html_url: string;
	url: string;
}
