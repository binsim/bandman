import prisma from '$lib/prisma';
import type { Prisma } from '../../../generated/prisma/client.js';

import type { PageServerLoad } from './$types.js';

export const load: PageServerLoad = async ({ params }) => {
	const songs = await prisma.song.findMany({
		include: {
			labels: false,
			userSongs: false,
			playlistSongs: false,
		},
	});

	return { songs } as { songs: Prisma.SongSelect };
};
