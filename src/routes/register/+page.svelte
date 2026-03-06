<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/Button.svelte';
	import Input from '$lib/components/Input.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let password = $state('');
	let repassword = $state('');

	let passwordMatchingError = $derived(
		password !== repassword && password !== '' && repassword !== ''
			? 'Passwords do not match'
			: '',
	);
</script>

<div class="login-container">
	<div class="login-card">
		<div class="login-header">
			<h2>Register</h2>
			<p>Enter credentials to create your account</p>
		</div>

		<form class="login-form" method="POST" use:enhance>
			<div class="form-group">
				<Input
					type="text"
					name="username"
					required
					value={form?.username ?? ''}
					autocomplete="username"
					label="Username"
					showErrorMessage={false}
					errorMessage={form?.errorMessage}
				/>
			</div>

			<div class="form-group">
				<Input
					type="password"
					name="password"
					label="Password"
					required
					autocomplete="current-password"
					bind:value={password}
					showErrorMessage={false}
					errorMessage={passwordMatchingError !== ''
						? passwordMatchingError
						: form?.errorMessage}
				/>
			</div>

			<div class="form-group">
				<Input
					type="password"
					name="repassword"
					label="Confirm Password"
					required
					autocomplete="current-password"
					bind:value={repassword}
					errorMessage={passwordMatchingError !== ''
						? passwordMatchingError
						: form?.errorMessage}
				/>
			</div>

			<Button text="Register" type="submit" />
		</form>

		<div class="signup-link">
			<p>Already have an account? <a href="/login">Sign In</a></p>
		</div>
	</div>
</div>

<style lang="scss">
	.login-container {
		width: 100%;
		max-width: 400px;
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
	}

	.login-card {
		background: white;
		border-radius: 12px;
		padding: 32px;
		border: 1px solid #e2e8f0;
		box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
	}

	.login-header {
		text-align: center;
		margin-bottom: 32px;
	}

	.login-header h2 {
		font-size: 1.875rem;
		font-weight: 700;
		color: #1e293b;
		margin-bottom: 8px;
	}

	.login-header p {
		color: #64748b;
		font-size: 0.875rem;
	}

	/* Form Styles */
	.form-group {
		margin-bottom: 20px;
	}

	/* Signup Link */
	.signup-link {
		text-align: center;
	}

	.signup-link p {
		color: #64748b;
		font-size: 0.875rem;
	}

	.signup-link a {
		color: #6366f1;
		text-decoration: none;
		font-weight: 500;
		transition: color 0.2s ease;
	}

	.signup-link a:hover {
		color: #4f46e5;
	}

	/* Mobile Responsive */
	@media (max-width: 480px) {
		.login-card {
			padding: 24px;
			margin: 10px;
		}

		.login-header h2 {
			font-size: 1.5rem;
		}
	}
</style>
