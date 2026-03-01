import { redirect, type Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
	const session = event.cookies.get('session');

	if (!session && event.url.pathname !== '/login' && event.url.pathname !== '/register') {
		return redirect(307, '/login');
	}

	return resolve(event);
};
