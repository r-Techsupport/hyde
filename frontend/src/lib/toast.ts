import { writable } from 'svelte/store';

interface ToastInfo {
	message: string;
	id?: number;
	type: ToastType;
	dismissible: boolean;
	timeout?: number;
}

export enum ToastType {
	Info = 'info',
	Error = 'error',
	Success = 'success',
	Warning = "Warning"
}

export const toasts = writable<ToastInfo[]>([]);

/**
 * Add a new toast notification to the ui
 * @param toast configure the message, id, type, whether or not it's dismissible, and the timeout if applicable.
 * @returns the id of the toast generated
 *
 * TODO: this doesn't need to take an object, the interface could be cleaned up a lot
 */
export function addToast(toast: ToastInfo): number {
	// Create a unique ID so we can easily find/remove it
	// if it is dismissible/has a timeout.
	const id = Math.floor(Math.random() * 10000);

	// Setup some sensible defaults for a toast.
	const defaults = {
		id,
		type: ToastType.Info,
		dismissible: true,
		timeout: 3000
	};

	// Push the toast to the top of the list of toasts
	toasts.update((all) => [{ ...defaults, ...toast }, ...all]);

	// If toast is dismissible, dismiss it after "timeout" amount of time.
	if (toast.timeout) setTimeout(() => dismissToast(id), toast.timeout);
	return id;
}

/**
 * Remove a toast notification from the ui
 * @param id The ID of the toast to remove, should have been returned by addToast.
 */
export function dismissToast(id: number) {
	toasts.update((all) => all.filter((t) => t.id !== id));
}
