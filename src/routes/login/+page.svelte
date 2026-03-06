<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/Button.svelte';
	import Input from '$lib/components/Input.svelte';
	import { delay } from '@vitest/utils/timers';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
</script>

<div class="login-container">
	<div class="login-card">
		<div class="login-header">
			<h2>Sign In</h2>
			<p>Enter your credentials to access your account</p>
		</div>

		<form class="login-form" method="POST" use:enhance>
			<div class="form-group">
				<Input
					type="text"
					name="username"
					label="Username"
					required
					value={form?.username ?? ''}
					autocomplete="username"
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
					errorMessage={form?.errorMessage}
				/>
			</div>

			<div class="form-options">
				<label class="remember-wrapper">
					<input type="checkbox" id="remember" name="remember" />
					<span class="checkbox-label">
						<span class="checkmark"></span>
						Remember me
					</span>
				</label>
				<a href="/password-forgotten" class="forgot-password">Forgot password?</a>
			</div>

			<Button text="Sign In" type="submit" />
		</form>

		<div class="signup-link">
			<p>Don't have an account? <a href="/register">Create one</a></p>
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

	/* Form Options */
	.form-options {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 24px;
		flex-wrap: wrap;
		gap: 12px;
	}

	.remember-wrapper {
		display: flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
	}

	.remember-wrapper input[type='checkbox'] {
		display: none;
	}

	.checkbox-label {
		color: #64748b;
		font-size: 0.875rem;
		cursor: pointer;
		user-select: none;
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.checkmark {
		width: 16px;
		height: 16px;
		border: 2px solid #d1d5db;
		border-radius: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.2s ease;
		flex-shrink: 0;
		background: white;
	}

	.remember-wrapper input[type='checkbox']:checked ~ .checkbox-label .checkmark {
		background: #6366f1;
		border-color: #6366f1;
	}

	.remember-wrapper input[type='checkbox']:checked ~ .checkbox-label .checkmark::after {
		content: '✓';
		color: white;
		font-size: 10px;
		font-weight: bold;
	}

	.forgot-password {
		color: #6366f1;
		text-decoration: none;
		font-size: 0.875rem;
		font-weight: 500;
		transition: color 0.2s ease;
	}

	.forgot-password:hover {
		color: #4f46e5;
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

		.form-options {
			flex-direction: column;
			align-items: flex-start;
			gap: 16px;
		}
	}
</style>
