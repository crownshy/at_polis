import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface User {
	did: string;
	handle: string;
	identifier: string;
	password: string;
}

// Initialize from localStorage if available
const storedUser = browser ? localStorage.getItem('user') : null;
const initialUser = storedUser ? JSON.parse(storedUser) : null;

export const user = writable<User | null>(initialUser);

// Subscribe to user changes and persist to localStorage
if (browser) {
	user.subscribe((value) => {
		if (value) {
			localStorage.setItem('user', JSON.stringify(value));
		} else {
			localStorage.removeItem('user');
		}
	});
}
