<script lang="ts">
	import type { FullAutoFill } from 'svelte/elements';

	interface Props {
		errorMessage?: string;
		showErrorMessage?: boolean;
		type: 'text' | 'password';
		name: string;
		value?: any;
		autocomplete?: FullAutoFill;
		label: string;
		required?: boolean | null | undefined;
	}
	let {
		errorMessage = undefined,
		showErrorMessage = true,
		name,
		value = $bindable(),
		autocomplete,
		label,
		required,
		type,
	}: Props = $props();

	let showPassword = $state(false);
</script>

<div
	class={'input-wrapper ' +
		(type === 'password' ? 'password-wrapper ' : '') +
		(errorMessage !== '' && errorMessage !== undefined ? 'error ' : '')}
>
	<input
		type={type === 'password' && showPassword ? 'text' : type}
		id={name}
		{name}
		{required}
		{autocomplete}
		bind:value
	/>
	<label for={name}>{label}</label>
	{#if type === 'password'}
		<button
			type="button"
			class="password-toggle"
			id="passwordToggle"
			aria-label="Toggle password visibility"
			onclick={() => (showPassword = !showPassword)}
		>
			<span class={'eye-icon ' + (showPassword ? 'show-password' : '')} aria-hidden="true"
			></span>
		</button>
	{/if}
</div>
{#if (errorMessage ?? '' !== '') && showErrorMessage}
	<span class="error-message" id="passwordError">{errorMessage}</span>
{/if}

<style lang="scss">
	.input-wrapper input {
		background: white;
		border: 2px solid #e2e8f0;
		border-radius: 8px;
		padding: 12px 16px 8px 16px;
		color: #1e293b;
		font-size: 16px;
		transition: all 0.2s ease;
		width: 100%;
		outline: none;
	}

	.input-wrapper input::placeholder {
		color: transparent;
	}

	.input-wrapper label {
		position: absolute;
		left: 16px;
		top: 12px;
		color: #64748b;
		font-size: 16px;
		transition: all 0.2s ease;
		pointer-events: none;
		transform-origin: left top;
	}

	.input-wrapper input:focus,
	.input-wrapper input:valid {
		border-color: #6366f1;
	}

	.input-wrapper input:focus + label,
	.input-wrapper input:valid + label {
		transform: translateY(-8px) scale(0.875);
		color: #6366f1;
		font-weight: 500;
	}

	.input-wrapper.error input:focus + label,
	.input-wrapper.error input:valid + label {
		color: #ef4444;
	}
	/* Password Toggle */
	.password-wrapper input {
		padding-right: 48px;
	}

	.password-toggle {
		position: absolute;
		right: 12px;
		top: 50%;
		transform: translateY(-50%);
		background: none;
		border: none;
		cursor: pointer;
		padding: 8px;
		color: #64748b;
		transition: color 0.2s ease;
	}

	.password-toggle:hover {
		color: #1e293b;
	}

	.eye-icon {
		display: block;
		width: 20px;
		height: 20px;
		background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%2364748b' stroke-width='1.5'%3e%3cpath stroke-linecap='round' stroke-linejoin='round' d='M15 12a3 3 0 11-6 0 3 3 0 016 0z'/%3e%3cpath stroke-linecap='round' stroke-linejoin='round' d='M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z'/%3e%3c/svg%3e");
		background-size: contain;
		background-repeat: no-repeat;
		background-position: center;
		transition: background-image 0.2s ease;
	}

	.eye-icon.show-password {
		background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%2364748b' stroke-width='1.5'%3e%3cpath stroke-linecap='round' stroke-linejoin='round' d='M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 11-4.243-4.243m4.242 4.242L9.88 9.88'/%3e%3c/svg%3e");
	}

	.input-wrapper {
		position: relative;
		display: flex;
		flex-direction: column;
	}

	.input-wrapper.error input {
		border-color: #ef4444;
	}

	.input-wrapper.error label {
		color: #ef4444;
	}
	/* Error Messages */
	.error-message {
		display: block;
		color: #ef4444;
		font-size: 0.75rem;
		font-weight: 500;
		margin-top: 4px;
		margin-left: 4px;
		transition: all 0.2s ease;
	}
</style>
