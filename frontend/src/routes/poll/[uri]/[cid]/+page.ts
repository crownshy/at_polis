import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ parent }) => {
	let data = await parent();
	return data
}

