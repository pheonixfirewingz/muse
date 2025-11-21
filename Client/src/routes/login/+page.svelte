<script lang="ts">
	import { goto } from '$app/navigation';
	import { apiService } from '$lib/api';

	let register: boolean = false;
	let username: string = '';
	let email: string = '';
	let password: string = '';
	let confirmPassword: string = '';
	let errors: Record<string, string> = {};
	let touched: Record<string, boolean> = {};
	let serverError: string = '';
	let isSubmitting: boolean = false;

	function validateUsername(value: string): string | null
	{
		if (!value) return 'Username is required';
		if (value.length < 3) return 'Username must be at least 3 characters';
		if (value.length > 20) return 'Username must be less than 20 characters';
		if (!/^[a-zA-Z0-9_]+$/.test(value)) return 'Username can only contain letters, numbers, and underscores';
		return null;
	}

	function validateEmail(value: string): string | null
	{
		if (!value) return 'Email is required';
		const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
		if (!emailRegex.test(value)) return 'Please enter a valid email address';
		return null;
	}

	function validatePassword(value: string): string | null
	{
		if (!value) return 'Password is required';
		if (value.length < 8) return 'Password must be at least 8 characters';
		if (value.length > 26) return 'Password must be less than 26 characters';
		if (!/[0-9]/.test(value)) return 'Password must contain at least one number';
		if (!/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]/.test(value)) return 'Password must contain at least one special character';
		return null;
	}

	function validateConfirmPassword(value: string): string | null
	{
		if (!value) return 'Please confirm your password';
		if (value !== password) return 'Passwords do not match';
		return null;
	}

	function validateForm(): boolean
	{
		errors = {};
		if (register)
		{
			const usernameError = validateUsername(username);
			const emailError = validateEmail(email);
			const passwordError = validatePassword(password);
			const confirmPasswordError = validateConfirmPassword(confirmPassword);
			if (usernameError) errors.username = usernameError;
			if (emailError) errors.email = emailError;
			if (passwordError) errors.password = passwordError;
			if (confirmPasswordError) errors.confirmPassword = confirmPasswordError;
		}
		else
		{
			const usernameError = validateUsername(username);
			const passwordError = password ? null : 'Password is required';
			if (usernameError) errors.username = usernameError;
			if (passwordError) errors.password = passwordError;
		}

		return Object.keys(errors).length === 0;
	}

	function handleBlur(field: string)
	{
		touched[field] = true;
		// Clear server error when user starts typing again
		serverError = '';

		switch(field)
		{
			case 'username':
			{
				const usernameError = validateUsername(username);
				if (usernameError) errors.username = usernameError;
				else delete errors.username;
				break;
			}
			case 'email':
			{
				const emailError = validateEmail(email);
				if (emailError) errors.email = emailError;
				else delete errors.email;
				break;
			}
			case 'password':
			{
				const passwordError = validatePassword(password);
				if (passwordError) errors.password = passwordError;
				else delete errors.password;
				if (touched.confirmPassword)
				{
					const confirmError = validateConfirmPassword(confirmPassword);
					if (confirmError) errors.confirmPassword = confirmError;
					else delete errors.confirmPassword;
				}
				break;
			}
			case 'confirmPassword':
			{
				const confirmError = validateConfirmPassword(confirmPassword);
				if (confirmError) errors.confirmPassword = confirmError;
				else delete errors.confirmPassword;
				break;
			}
		}
		errors = errors;
	}

	async function handleLogin(e: Event)
	{
		e.preventDefault();
		touched = { username: true, password: true };
		serverError = '';

		if (validateForm())
		{
			isSubmitting = true;
			try
			{
				const response = await apiService.login({ username, password });
				if (response.success && response.data?.token)
				{
					await goto('/app');
				}
				else
				{
					serverError = response.message || 'Login failed';
				}
			}
			catch (error)
			{
				if (error instanceof Error)
				{
					try
					{
						const errorResponse = JSON.parse(error.message);
						serverError = errorResponse.message || 'Login failed';
						if (errorResponse.errors)
						{
							errors = { ...errors, ...errorResponse.errors };
						}
					}
					catch
					{
						serverError = error.message;
					}
				}
				else
				{
					serverError = 'An error occurred during login';
				}
			}
			finally
			{
				isSubmitting = false;
			}
		}
	}

	async function handleRegister(e: Event)
	{
		e.preventDefault();
		touched = { username: true, email: true, password: true, confirmPassword: true };
		serverError = '';

		if (validateForm())
		{
			isSubmitting = true;
			try
			{
				const response = await apiService.register({ username, email, password, confirm_password: confirmPassword });
				if (response.success && response.data?.token)
				{
					await goto('/app');
				}
				else
				{
					serverError = response.message || 'Registration failed';
				}
			}
			catch (error)
			{
				if (error instanceof Error)
				{
					try
					{
						const errorResponse = JSON.parse(error.message);
						serverError = errorResponse.message || 'Registration failed';
						if (errorResponse.errors)
						{
							const mappedErrors = { ...errorResponse.errors };
							if (mappedErrors.confirm_password)
							{
								mappedErrors.confirmPassword = mappedErrors.confirm_password;
								delete mappedErrors.confirm_password;
							}
							errors = { ...errors, ...mappedErrors };
						}
					}
					catch
					{
						serverError = error.message;
					}
				}
				else
				{
					serverError = 'An error occurred during registration';
				}
			}
			finally
			{
				isSubmitting = false;
			}
		}
	}

	function toggleMode()
	{
		register = !register;
		username = '';
		email = '';
		password = '';
		confirmPassword = '';
		errors = {};
		touched = {};
		serverError = '';
	}
</script>
<div class="h-screen w-screen flex flex-col justify-center items-center">
	<div class="card p-8" style="width: calc(100vw / 2); max-width: 500px; min-width: 400px;">
		{#if register}
			<form class="flex flex-col gap-4" on:submit={handleRegister}>
				<h1 class="text-3xl font-bold text-center mb-4">Create Account</h1>

				{#if serverError}
					<div class="alert variant-filled-error">
						<span>{serverError}</span>
					</div>
				{/if}

				<input type="text" bind:value={username} on:blur={() => handleBlur('username')}
							 placeholder="Enter username" class="input"
							 class:input-error={touched.username && errors.username}
							 disabled={isSubmitting}/>
				{#if touched.username && errors.username}
					<span class="text-error-500 text-sm -mt-2">{errors.username}</span>
				{/if}
				<input type="email" bind:value={email}
							 on:blur={() => handleBlur('email')}
							 placeholder="Enter email" class="input"
							 class:input-error={touched.email && errors.email}
							 disabled={isSubmitting}
				/>
				{#if touched.email && errors.email}
					<span class="text-error-500 text-sm -mt-2">{errors.email}</span>
				{/if}
				<input type="password" bind:value={password}
							 on:blur={() => handleBlur('password')}
							 placeholder="Enter password" class="input"
							 class:input-error={touched.password && errors.password}
							 disabled={isSubmitting}
				/>
				{#if touched.password && errors.password}
					<span class="text-error-500 text-sm -mt-2">{errors.password}</span>
				{:else if touched.password}
					<span class="text-surface-600 text-xs -mt-2">8-26 characters, at least one number and special character</span>
				{/if}
				<input type="password" bind:value={confirmPassword}
							 on:blur={() => handleBlur('confirmPassword')}
							 placeholder="Confirm password" class="input"
							 class:input-error={touched.confirmPassword && errors.confirmPassword}
							 disabled={isSubmitting}
				/>
				{#if touched.confirmPassword && errors.confirmPassword}
					<span class="text-error-500 text-sm -mt-2">{errors.confirmPassword}</span>
				{/if}
				<div class="flex flex-col gap-2 mt-2">
					<button type="submit" class="btn variant-filled-primary w-full" disabled={isSubmitting}>
						{isSubmitting ? 'Registering...' : 'Register'}
					</button>
					<div class="flex items-center justify-center gap-2">
						<h2 class="text-surface-300">Already have an account?</h2>
						<button type="button" class="underline" on:click={toggleMode} disabled={isSubmitting}>
							Login
						</button>
					</div>
				</div>
			</form>
		{:else}
			<form class="flex flex-col gap-4" on:submit={handleLogin}>
				<h1 class="text-3xl font-bold text-center mb-4">Login</h1>

				{#if serverError}
					<div class="alert variant-filled-error">
						<span>{serverError}</span>
					</div>
				{/if}

				<input
					type="text"
					bind:value={username}
					on:blur={() => handleBlur('username')}
					placeholder="Enter username"
					class="input"
					class:input-error={touched.username && errors.username}
					disabled={isSubmitting}
				/>
				{#if touched.username && errors.username}
					<span class="text-error-500 text-sm -mt-2">{errors.username}</span>
				{/if}

				<input
					type="password"
					bind:value={password}
					on:blur={() => handleBlur('password')}
					placeholder="Enter password"
					class="input"
					class:input-error={touched.password && errors.password}
					disabled={isSubmitting}
				/>
				{#if touched.password && errors.password}
					<span class="text-error-500 text-sm -mt-2">{errors.password}</span>
				{/if}

				<div class="flex flex-col gap-2 mt-2">
					<button type="submit" class="btn variant-filled-primary w-full" disabled={isSubmitting}>
						{isSubmitting ? 'Logging in...' : 'Login'}
					</button>
					<div class="flex items-center justify-center gap-2">
						<h2 class="text-surface-300">Don't have an account?</h2>
						<button type="button" class="underline" on:click={toggleMode} disabled={isSubmitting}>
							Register
						</button>
					</div>
				</div>
			</form>
		{/if}
	</div>
</div>