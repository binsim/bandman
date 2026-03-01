import { redirect, type RequestHandler } from '@sveltejs/kit';

export const POST: RequestHandler = async ({ cookies }) => {
	// Remove the session cookie set on login
	cookies.delete('session', { path: '/' });

	// Redirect the user to the login page after logout
	throw redirect(302, '/login');
};
