<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PageProps } from './$types';
	import { setLocale, LL } from '$lib/i18n/i18n-svelte';

	let { data, form }: PageProps = $props();
</script>

<form method="POST" use:enhance>
	<input type="hidden" name="id" value={form?.id ?? ''} />
	<label>{$LL.Title} <input name="title" type="text" value={form?.title ?? ''} required /></label>
	<label
		>{$LL.Artist} <input name="artist" type="text" value={form?.artist ?? ''} required /></label
	>
	<label>{$LL.Duration} <input name="duration" type="time" value={form?.duration ?? ''} /></label>
	<label>{$LL.Label} <input name="labels" value={form?.labels ?? ''} /></label>

	<button type="submit">Submit</button>
	<button>Cancel</button>
</form>
