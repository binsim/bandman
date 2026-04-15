<script lang="ts">
	import type { PageProps } from './$types';
	let { data }: PageProps = $props();

	let { wishItems, usersForVoting } = $derived(data);
</script>

<h1>Wishlist</h1>

<table>
	<thead>
		<tr>
			<th rowspan="2">Title</th>
			<th rowspan="2">Artist</th>
			<th rowspan="2">Link</th>

			<th colspan={usersForVoting.length}>Voting</th>
		</tr>
		<tr>
			{#each usersForVoting as user}
				<th>{user.name}</th>
			{/each}
		</tr>
	</thead>

	<tbody>
		{#each wishItems as item}
			<tr>
				<td>{item.title}</td>
				<td>{item.artist}</td>
				<td>{item.link}</td>
				{#each item.votings as voting}
					{#if voting.canEdit}
						<td><input type="checkbox" /></td>
					{:else}
						<td>{voting.vote?.option ?? '-'}</td>
					{/if}
				{/each}
			</tr>
		{/each}
	</tbody>
</table>

<style lang="scss">
	table {
		width: 100%;
		table-layout: fixed;
		border: 1px solid rgba(255, 255, 255, 0.4);
		border-collapse: collapse;
	}

	thead {
		background-color: rgba(255, 255, 255, 0.3);

		tr:first-child {
			text-transform: uppercase;
		}
	}

	tbody {
		overflow: auto;

		tr {
			&:nth-child(even) {
				background-color: rgba(255, 255, 255, 0.1);
			}
		}
	}

	th {
		text-align: center;
		border: 1px solid rgba(255, 255, 255, 0.4);
		border-collapse: collapse;
	}

	td {
		padding: 0.5rem;
		border: 1px solid rgba(255, 255, 255, 0.4);
		border-collapse: collapse;
	}
</style>
