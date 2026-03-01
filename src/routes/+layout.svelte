<script lang="ts">
	import { page } from '$app/state';
	import favicon from '$lib/assets/favicon.svg';

	async function logout() {
		const response = await fetch('/', {
			method: 'POST',
			credentials: 'same-origin',
			headers: {
				'Content-Type': 'application/json',
			},
		});

		if (!response.ok) {
			throw new Error('Failed to logout');
		}

		location.href = '/login';
	}

	let { children } = $props();
</script>

<svelte:head>
	<meta name="color-scheme" content="light dark" />
	<link rel="icon" href={favicon} />
</svelte:head>

{#if page.url.pathname !== '/login' && page.url.pathname !== '/register'}
	<header>
		<nav>
			<li><a href="/">Home</a></li>
		</nav>

		<div class="logout">
			<button onclick={logout}>Logout</button>
		</div>
	</header>
{/if}

<main>
	{@render children()}
</main>

{#if page.url.pathname !== '/login' && page.url.pathname !== '/register'}
	<footer></footer>
{/if}

<style lang="scss">
	header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
</style>
