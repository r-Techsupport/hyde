import { writable } from 'svelte/store';

export const currentFile = writable('');

export enum Permissions {
	ManageUsers = 'manage_users',
	ManageContent = 'manage_content'
}
