<script lang="ts">
	interface Props {
		text: string;
		type: 'button' | 'submit' | 'reset';
		onclick?: () => Promise<void>;
	}

	let { text, type, onclick }: Props = $props();

	let isLoading = $state();
</script>

<button
	{type}
	class={'login-btn ' + (isLoading ? 'loading' : '')}
	onclick={async () => {
		if (onclick !== undefined) {
			isLoading = true;
			await onclick();
			isLoading = false;
		}
	}}
>
	<span class="btn-text">{text}</span>
	<span class="btn-loader"></span>
</button>

<style lang="scss">
	/* Button */
	.login-btn {
		width: 100%;
		background: #6366f1;
		border: none;
		border-radius: 8px;
		padding: 12px 24px;
		color: white;
		font-size: 16px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s ease;
		position: relative;
		margin-bottom: 24px;
	}

	.login-btn:hover {
		background: #4f46e5;
	}

	.login-btn:active {
		transform: translateY(1px);
	}

	/* Loading State */
	.login-btn.loading {
		pointer-events: none;
		background: #a5a6f6;
	}

	.btn-text {
		transition: opacity 0.2s ease;
	}

	.btn-loader {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 16px;
		height: 16px;
		border: 2px solid transparent;
		border-top: 2px solid white;
		border-radius: 50%;
		opacity: 0;
		animation: spin 1s linear infinite;
		transition: opacity 0.2s ease;
	}

	.login-btn.loading .btn-text {
		opacity: 0;
	}

	.login-btn.loading .btn-loader {
		opacity: 1;
	}

	/* Animations */
	@keyframes spin {
		0% {
			transform: translate(-50%, -50%) rotate(0deg);
		}
		100% {
			transform: translate(-50%, -50%) rotate(360deg);
		}
	}
</style>
