import prisma from '$lib/prisma';
import { fail } from '@sveltejs/kit';
import type { Prisma } from '../../../generated/prisma/client.js';

import type { Actions, PageServerLoad } from './$types.js';

export const load: PageServerLoad = async () => {
	return {
		form: {},
	};
};

export const actions = {
	default: async ({ cookies, request }) => {
		const data = await request.formData();

		const id = data.get('id');
		const title = data.get('title');
		const artist = data.get('artist');
		const duration = data.get('duration');
		const labels = data.get('labels');

		return fail(400, {
			id,
			title,
			artist,
			duration,
			labels,
		});
	},
} satisfies Actions;
