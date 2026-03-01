import prisma from '$lib/prisma';
import type { Actions, PageServerLoad } from './$types';
import { fail, redirect } from '@sveltejs/kit';
import { compare } from 'bcrypt-ts';

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
		const remember = formData.get('remember');

		if (
			typeof username !== 'string' ||
			typeof password !== 'string' ||
			username === '' ||
			password === ''
		) {
			console.log('username and password not provided');
			return fail(400, { username, remember, errorMessage: 'Invalid username or password' });
		}

		const user = await prisma.user.findUnique({
			where: {
				name: username,
			},
		});

		if (user == null) {
			console.log(`user ${username} not found`);
			return fail(400, { username, remember, errorMessage: 'Invalid username or password' });
		}

		if (!(await compare(password, user.passwordHash))) {
			console.log(`password for user ${username} does not match`);
			return fail(400, { username, remember, errorMessage: 'Invalid username or password' });
		}

		cookies.set('session', user.id, {
			path: '/',
			httpOnly: true,
			sameSite: 'strict',
			maxAge: remember ? 60 * 60 * 24 * 7 : 60 * 60,
		});

		console.log(`login successful for user ${username}`);
		return redirect(302, '/');
	},
} satisfies Actions;
