/**
 * @file
 * Misc type declarations
 */

/**
 * All configurable permissions
 */
export enum Permission {
	ManageUsers = 'ManageUsers',

	// Reads
	GetDoc = 'GetDoc',
	GetAsset = 'GetAsset',
	GetDocTree = 'GetDocTree',
	GetAssetTree = 'GetAssetTree',
	GetCurrentBranch = 'GetCurrentBranch',

	// Content mutation
	PutDoc = 'PutDoc',
	PutAsset = 'PutAsset',
	DeleteDoc = 'DeleteDoc',
	DeleteAsset = 'DeleteAsset',

	// Version control
	GitAdd = 'GitAdd',
	GitCommit = 'GitCommit',
	GitPush = 'GitPush',
	Pull = 'Pull',
	GitPullBranch = 'GitPullBranch',
	CheckoutOrCreateBranch = 'CheckoutOrCreateBranch',
	Reclone = 'Reclone'
}

/**
 * A map between the internal permission representations and the "pretty print" representation
 */
export const allPermissions: Map<Permission, string> = new Map();
allPermissions.set(Permission.ManageUsers, 'Manage Users');

allPermissions.set(Permission.GetDoc, 'Read Document');
allPermissions.set(Permission.GetAsset, 'Read Asset');
allPermissions.set(Permission.GetDocTree, 'Read Document Tree');
allPermissions.set(Permission.GetAssetTree, 'Read Asset Tree');
allPermissions.set(Permission.GetCurrentBranch, 'Read Current Branch');

allPermissions.set(Permission.PutDoc, 'Create/Edit Document');
allPermissions.set(Permission.PutAsset, 'Create/Edit Asset');
allPermissions.set(Permission.DeleteDoc, 'Delete Document');
allPermissions.set(Permission.DeleteAsset, 'Delete Asset');

allPermissions.set(Permission.GitAdd, 'Stage Changes');
allPermissions.set(Permission.GitCommit, 'Commit Changes');
allPermissions.set(Permission.GitPush, 'Push Changes');
allPermissions.set(Permission.Pull, 'Pull Latest Changes');
allPermissions.set(Permission.GitPullBranch, 'Pull Branch');
allPermissions.set(Permission.CheckoutOrCreateBranch, 'Checkout/Create Branch');
allPermissions.set(Permission.Reclone, 'Reclone Repository');

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
