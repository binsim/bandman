import { defaultExternalConditions } from 'vite';
import type { BaseTranslation } from '../i18n-types';

const en = {
	Title: 'Title',
	Artist: 'Artist',
	Duration: 'Duration',
	Label: 'Label',
	Logout: 'Logout',
	SignIn: 'Sign In',
	Register: 'Register',
	EnterCredentialsForAccountAccess: 'Enter credentials to access your account.',
	Username: 'Username',
	Password: 'Password',
	ConfirmPassword: 'Confirm Password',
	PasswordMismatch: 'Passwords do not match.',
	RememberMe: 'Remember me',
	ForgotPassword: 'Forgot password?',
	DontHaveAnAccount: "Don't have an account?",
	AlreadyHaveAnAccount: 'Already have an account?',
} satisfies BaseTranslation;

export default en;
