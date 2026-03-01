import prisma from '$lib/prisma';
import type { Actions, PageServerLoad } from './$types';
import { fail, redirect } from '@sveltejs/kit';
import { compare, hash } from 'bcrypt-ts';

export const load: PageServerLoad = async ({ cookies }) => {
	return {
		form: {},
	};
};

export const actions = {
	default: async ({ cookies, request }) => {
		const formData = await request.formData();

		const username = formData.get('username');
		const password = formData.get('password');

		if (
			typeof username !== 'string' ||
			typeof password !== 'string' ||
			username === '' ||
			password === ''
		) {
			console.log('username and password not provided');
			return fail(400, { username, errorMessage: 'Invalid username or password' });
		}

		try {
			const user = await prisma.user.create({
				data: {
					name: username,
					passwordHash: await hash(password, 10),
				},
				include: {
					roles: true,
				},
			});

			console.log(
				`user ${username} has successfully been registered with rolse ${user.roles.join(',')}`,
			);
			return redirect(302, '/login');
		} catch {
			console.log(`user ${username} already exists`);
			return fail(400, { username, errorMessage: 'Username already exists' });
		}
	},
} satisfies Actions;
