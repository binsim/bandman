<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
</script>

<form method="POST" use:enhance>
	<input type="hidden" name="id" value={form?.id ?? ''} />
	<label>Title <input name="title" type="text" value={form?.title ?? ''} required /></label>
	<label>Artist <input name="artist" type="text" value={form?.artist ?? ''} required /></label>
	<label>Duration <input name="duration" type="time" value={form?.duration ?? ''} /></label>
	<label>Labels <input name="labels" value={form?.labels ?? ''} /></label>

	<button type="submit">Submit</button>
	<button>Cancel</button>
</form>
