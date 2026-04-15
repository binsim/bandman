import prisma from '$lib/prisma';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies }) => {
	const usersForVoting: { user_id: string; name: string; canEdit: boolean }[] = (
		await prisma.user.findMany()
	).map((user) => ({
		user_id: user.id,
		name: user.name ?? '',
		canEdit: user.id === cookies.get('session'),
	}));

	const wishItems = (
		await prisma.wishlistSong.findMany({
			include: { votings: true },
			orderBy: { title: 'asc' },
		})
	).map((item) => ({
		id: item.id,
		title: item.title,
		artist: item.artist,
		comment: item.comment,
		link: item.link,
		votings: usersForVoting.map((user) => ({
			user_id: user.user_id,
			vote: item.votings.find((v) => v.userId === user.user_id),
			canEdit: user.canEdit,
		})),
	}));

	console.log(wishItems, usersForVoting);

	return {
		wishItems,
		usersForVoting,
	};
};
