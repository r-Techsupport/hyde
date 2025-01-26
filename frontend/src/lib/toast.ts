/**
 * @file
 * The internals of the toast notification system
 */

import { writable } from 'svelte/store';

/**
 * Passed to `addToast`
 */
interface ToastInfo {
	/**
	 * The text displayed in the toast
	 */
	message: string;
	/**
	 * Optionally supplied, the ID to give the toast
	 */
	id?: number;
	/**
	 * The type of the toast (info, error, warning)
	 */
	type: ToastType;
	/**
	 * Whether or not the user can make the toast go away
	 */
	dismissible: boolean;
	/**
	 * How long before the toast fades away by itself
	 */
	timeout?: number;
}

/**
 * The category of notification to be displayed
 */
export enum ToastType {
	Info = 'info',
	Error = 'error',
	Success = 'success',
	Warning = 'warning'
}

export const toasts = writable<ToastInfo[]>([]);

/**
 * Add a new toast notification to the UI.
 *
 * @param message The string rendered inside of the toast notif
 * @param type The category (ToastType.Info, ToastType.Warning, et cetera)
 * @param dismissible Whether or not the user can dismiss a toast with a button
 * @param timeout The number of milliseconds before the toast goes away
 * @returns The ID of the newly created toast, to be used in conjunction with `dismissToast`
 *
 * @example
 * ```ts
 * // Create an info toast that's dismissable by the user, or after 3000ms
 * addToast("Hello, world");
 * // Create a warning toast the user cannot dismiss
 * addToast("Warning...", ToastType.Warning, false);
 * // Create an error toast that goes away after 500ms, or is dismissable by the user
 * addToast("Error...", ToastType.Error, true, 500);
 * ```
 */
export function addToast(
	message: string,
	type = ToastType.Info,
	dismissible = true,
	timeout = 3000
) {
	// Create a uniquee ID to identify dismissable/timed out toasts
	const id = Math.floor(Math.random() * 10000);

	// Push the toast to the top of the list of toasts
	toasts.update((all) => [{ id, type, message, dismissible, timeout }, ...all]);
	// If toast is dismissible, dismiss it after "timeout" amount of time.
	if (dismissible) setTimeout(() => dismissToast(id), timeout);
	return id;
}

/**
 * Remove a toast notification from the ui
 * @param id The ID of the toast to remove, should have been returned by addToast.
 */
export function dismissToast(id: number) {
	toasts.update((all) => all.filter((t) => t.id !== id));
}
