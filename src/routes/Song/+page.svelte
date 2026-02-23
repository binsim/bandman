<script lang="ts">
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

{#each data.songs as song}
	<div>
		<h2>{song.title}</h2>
		<p>{song.artist}</p>
	</div>
{/each}
