import { writable, type Writable } from 'svelte/store';
import type { User, INode, Branch } from './types';
export const currentFile = writable('');
import { dev } from '$app/environment';

/** The type of media currently being edited */
export enum SelectedMode {
	Documents,
	Assets
}

/**
 * The user object for the currently logged in user, or a default with an ID of -1
 * if it hasn't been loaded in yet
 */
export const me: Writable<User> = writable({
	id: -1,
	username: 'Loading..',
	avatar_url: 'https://cdn.discordapp.com/embed/avatars/0.png',
	groups: [],
	permissions: []
});

/** The currently selected branch */
export const branchName: Writable<string> = writable('');
/** The branch pull requests merge into */
export const baseBranch: Writable<string> = writable('');
/** A list of every branch */
export const allBranches = writable<Branch[]>([]);
/** The text currently in the input editor */
export const editorText = writable<string>('');

/**
 * The filesystem tree for the document folder
 */
export const documentTree: Writable<INode> = writable({
	name: '',
	children: []
});

export const assetTree: Writable<INode> = writable({
	name: '',
	children: []
});

export let apiAddress = '';

// Set the API url to localhost for supporting
// dev environments
if (dev) {
	console.log('Development environment detected, switching to local server');
	apiAddress = 'http://localhost:8080';
}
