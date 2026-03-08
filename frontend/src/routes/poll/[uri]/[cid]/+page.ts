import type { PageLoad } from './$types';

interface Poll {
	uri: string;
	did: string;
	cid: string;
	topic: string;
	description: string | null;
	createdAt: string;
	closedAt: string | null;
}

export const load: PageLoad = async ({ params, parent, fetch }) => {
	const data = await parent();
	const pollUri = decodeURIComponent(params.uri);

	// Fetch poll details
	const response = await fetch(`/api/polls/${encodeURIComponent(pollUri)}`);

	if (!response.ok) {
		throw new Error('Failed to load poll');
	}

	const poll: Poll | null = await response.json();

	return {
		...data,
		poll
	};
};

