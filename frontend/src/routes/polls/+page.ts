import type { PageLoad } from './$types';

export interface Poll {
	uri: string;
	did: string;
	cid: string;
	topic: string;
	description: string | null;
	createdAt: string;
	closedAt: string | null;
}

export const load: PageLoad = async ({ fetch }) => {
	const response = await fetch('/api/polls');

	if (!response.ok) {
		throw new Error('Failed to load polls');
	}

	const polls: Poll[] = await response.json();

	return {
		polls
	};
};
