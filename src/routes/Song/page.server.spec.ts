import { describe, it, expect } from 'vitest';
import { actions } from './+page.server';

describe('Song actions (server)', () => {
	const runDefault = async (formValues: Record<string, string | null>) => {
		// Minimal mock of the request object used by the action.
		// The action only calls `request.formData()` and then uses `.get(...)`.
		const mockRequest = {
			formData: async () => ({
				get: (name: string) =>
					Object.prototype.hasOwnProperty.call(formValues, name)
						? formValues[name]
						: null,
			}),
		};

		// actions.default receives an object with `cookies` and `request`
		// we pass a minimal cookies object (not used by the implementation).
		// use `any` for the argument shape to avoid tight typing in the test.
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		return await (actions as any).default({ cookies: {}, request: mockRequest } as any);
	};

	it('returns a failed action result (status 400) and echoes submitted fields', async () => {
		const payload = {
			id: '123',
			title: 'Test Song',
			artist: 'Unit Tester',
			duration: '00:03:30',
			labels: 'rock,indie',
		};

		const result = await runDefault(payload);

		// The server action uses `fail(400, {...})` so we expect a status of 400
		// and the returned payload to contain the same fields we submitted.
		expect(result).toBeTypeOf('object');
		// status is provided by SvelteKit's `fail`
		// (assert existence & value rather than relying on full shape)
		expect((result as any).status).toBe(400);

		// The rest of the properties should mirror the submitted form fields.
		expect((result as any).data.id).toBe(payload.id);
		expect((result as any).data.title).toBe(payload.title);
		expect((result as any).data.artist).toBe(payload.artist);
		expect((result as any).data.duration).toBe(payload.duration);
		expect((result as any).data.labels).toBe(payload.labels);
	});

	it('handles missing fields by returning null for absent values', async () => {
		const payload: Record<string, string | null> = {
			// intentionally omit most fields
			title: 'Only Title',
			// id, artist, duration, labels are missing -> should be null
		};

		const result = await runDefault(payload);

		expect((result as any).status).toBe(400);
		expect((result as any).data.title).toBe('Only Title');
		expect((result as any).data.id).toBeNull();
		expect((result as any).data.artist).toBeNull();
		expect((result as any).data.duration).toBeNull();
		expect((result as any).data.labels).toBeNull();
	});
});
