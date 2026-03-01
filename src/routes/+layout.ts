import type { LayoutLoad } from './$types';
import { loadLocale } from '$lib/i18n/i18n-util.sync';
import LL, { setLocale } from '$lib/i18n/i18n-svelte';
import type { Locales } from '$lib/i18n/i18n-types';
import { baseLocale } from '$lib/i18n/i18n-util';

export const prerender = false;

export const load: LayoutLoad = ({ url }) => {
	// const lang = (url.pathname.split('/').filter(Boolean)[0] as Locales) ?? baseLocale;
	const lang = baseLocale;

	loadLocale(lang);
	setLocale(lang);

	console.log(LL);
};
