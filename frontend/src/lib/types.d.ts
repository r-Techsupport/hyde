export enum Permission {
	ManageUsers = 'ManageUsers',
	ManageContent = 'ManageContent'
}

export interface User {
	id: number;
	username: string;
	groups: Group[];
	permissions: Permission[];
}

export interface Group {
	id: number;
	name: string;
}
