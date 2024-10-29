import { writable, type Writable } from 'svelte/store';
import type { User, INode } from './types';

export const currentFile = writable('');

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

/**
 * New store for branch name
 */
export const branchName: Writable<string> = writable('Set Branch'); // Default branch name
/**
 * New store for the tree
 */
export const documentTreeStore = writable<INode>({
	name: '',
	children: []
});

/**
 * New store for the page
 */
export const editorText = writable<string>('');
