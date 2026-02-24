import { page } from 'vitest/browser';
import { describe, it, expect } from 'vitest';
import { render } from 'vitest-browser-svelte';
import SongPage from './+page.svelte';

describe('Song page component (client)', () => {
	it('renders form fields and buttons', async () => {
		// Render the Svelte page component into the test browser page
		render(SongPage);

		// Assert form fields are present by their labels
		const title = page.getByLabelText('Title');
		await expect.element(title).toBeInTheDocument();

		const artist = page.getByLabelText('Artist');
		await expect.element(artist).toBeInTheDocument();

		const duration = page.getByLabelText('Duration');
		await expect.element(duration).toBeInTheDocument();

		const labels = page.getByLabelText('Labels');
		await expect.element(labels).toBeInTheDocument();

		// Assert submit button exists
		const submit = page.getByRole('button', { name: /submit/i });
		await expect.element(submit).toBeInTheDocument();
	});
});
