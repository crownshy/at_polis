import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ fetch }) => {
	try {
		console.log("Fetching ")
		const response = await fetch('/api/me', {
			credentials: 'include',
		});

		console.log("Fetching ", response)
		if (response.ok) {
			const data = await response.json();
			console.log({ data })
			return {
				authenticated: data.authenticated,
				did: data.did,
			};
		}
	} catch (error) {
		console.error('Failed to check authentication:', error);
	}

	return {
		authenticated: false,
		did: null,
	};
};

