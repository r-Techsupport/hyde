import { apiAddress } from "$lib/main";
import { addToast, ToastType } from "$lib/toast";

export interface Branch {
	name: string;
	isProtected: boolean;
}

interface BranchInfo {
    current: string,
    base: string,
    list: Branch[],
};

export const branchInfo: BranchInfo = $state({
    /** The name of the currently selected branch */
    current: '',
    /** The name of the repository's base branch */
    base: '',
    /** A list of every branch  */
    list: []
})


	/**
	 * Fetches the list of branches from the GitHub API, categorizing them as protected or non-protected.
	 *
	 * This asynchronous function sends a GET request to the specified GitHub repository's API endpoint
	 * to retrieve the branches. The function handles both successful and failed requests:
	 * - On success, it returns a promise that resolves to an object containing two arrays:
	 *   - `nonProtectedBranches`: A list of branches that are not protected.
	 *   - `protectedBranches`: A list of branches that are marked as protected.
	 * - On failure, it returns an object with empty arrays for both `nonProtectedBranches` and `protectedBranches`.
	 *
	 * @returns {Promise<{ nonProtectedBranches: Branch[]; protectedBranches: Branch[]; }>} A promise that resolves to an object with two properties:
	 * - `nonProtectedBranches`: An array of non-protected branches.
	 * - `protectedBranches`: An array of protected branches.
	 * If the request fails, both arrays will be empty.
	 *
	 * @throws {Error} Will throw an error if the response from the API is unsuccessful, with details including the status code and error message from the server.
	 *
	 * @example
	 * async function main() {
	 *   try {
	 *     const { nonProtectedBranches, protectedBranches } = await fetchExistingBranches();
	 *   } catch (err) {
	 *     console.error('Error fetching branches:', err);
	 *   }
	 * }
	 */
	async function fetchExistingBranches(): Promise<Branch[]> {
		const response = await fetch(`${apiAddress}/api/branches`, {
			method: 'GET',
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json'
			}
		});

		// Check if the response is successful
		if (!response.ok) {
			const errorMessage = await response.json();
			console.error('Failed to fetch branches:', errorMessage);
			addToast(
				`Error fetching branches: ${response.statusText}. ${JSON.stringify(errorMessage)}`,
				ToastType.Error
			);
			return [];
		}

		const data = await response.json();
		return (
			data.data?.branches?.map((branch: string) => ({
				name: branch.split(' (')[0],
				isProtected: branch.includes('(protected)')
			})) ?? []
		);
	}

	async function fetchDefaultBranch() {
		const response = await fetch(`${apiAddress}/api/repos/default-branch`);

		if (response.ok) {
			const data = await response.json();
			const defaultBranch = data.data;

			// Set the default branch to the baseBranch store
			branchInfo.base = defaultBranch;
		} else {
			console.error('Failed to fetch default branch:', response.statusText);
		}
	}

	async function fetchCurrentBranch() {
		try {
			const response = await fetch(`${apiAddress}/api/current-branch`);

			if (response.ok) {
				const data = await response.json();
				const currentBranch = data.data;
				branchInfo.current = currentBranch;
			} else {
				console.error('Failed to fetch current branch:', response.statusText);
			}
		} catch (error) {
			console.error('Error fetching current branch:', error);
		}
	}

	export async function loadBranchInfo() {
		fetchDefaultBranch();
		fetchCurrentBranch();
		const branches = await fetchExistingBranches();
		branchInfo.list = branches;
	};