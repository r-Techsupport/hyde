import { writable, type Writable } from 'svelte/store';
import type { User } from './types';

export const currentFile = writable('');

/**
 * The user object for the currently logged in user, or a default with an ID of -1
 * if it hasn't been loaded in yet
 */
export const me: Writable<User> = writable({
    id: -1,
    username: "Loading..",
    avatar_url: "https://cdn.discordapp.com/embed/avatars/0.png",
    groups: [],
    permissions: [],
});