<script lang="ts">
	import { goto } from '$app/navigation';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { user } from '$lib/stores/auth';
	import { login } from '$lib/api/client';

	let identifier = $state('');
	let password = $state('');
	let loading = $state(false);
	let error = $state('');
	let useOAuth = $state(true);
	let oauthHandle = $state('');

	async function handleOAuthLogin() {
		loading = true;
		error = '';

		try {
			// Strip @ prefix if present
			const cleanHandle = oauthHandle.startsWith('@') ? oauthHandle.slice(1) : oauthHandle;

			console.log('Initiating OAuth for handle:', cleanHandle);

			const response = await fetch('http://localhost:3000/oauth/authorize', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({ handle: cleanHandle }),
			});

			console.log('OAuth response status:', response.status);

			if (!response.ok) {
				let errorMessage = 'Failed to initiate OAuth: ' + response.status;
				try {
					const errorData = await response.json();
					console.error('OAuth initiation failed:', errorData);
					if (errorData.message) {
						errorMessage = errorData.message;
					}
				} catch (e) {
					const errorText = await response.text();
					console.error('OAuth initiation failed:', errorText);
					if (errorText) {
						errorMessage = errorText;
					}
				}
				throw new Error(errorMessage);
			}

			const data = await response.json();
			console.log('OAuth response data:', data);

			if (!data.authorization_url) {
				throw new Error('No authorization_url in response');
			}

			console.log('Redirecting to:', data.authorization_url);

			// Redirect to authorization URL
			window.location.href = data.authorization_url;
		} catch (e) {
			console.error('OAuth error:', e);
			error = 'Failed to initiate OAuth login: ' + (e instanceof Error ? e.message : 'Unknown error');
			loading = false;
		}
	}

	async function handleLogin() {
		loading = true;
		error = '';

		try {
			const response = await login(identifier, password);

			if (response.success && response.session) {
				user.set({
					did: response.session.did,
					handle: response.session.handle,
					identifier,
					password
				});
				goto('/');
			} else {
				error = response.message || 'Login failed';
			}
		} catch (e) {
			error = 'Failed to connect to server';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex items-center justify-center min-h-[60vh]">
	<Card class="w-full max-w-md p-6">
		<h1 class="text-2xl font-bold mb-6">Login to ATProto Polis</h1>

		<!-- Login method toggle -->
		<div class="flex gap-2 mb-6 p-1 bg-muted rounded-lg">
			<button
				type="button"
				class="flex-1 px-4 py-2 rounded-md text-sm font-medium transition-colors {useOAuth
					? 'bg-background shadow-sm'
					: 'hover:bg-background/50'}"
				onclick={() => (useOAuth = true)}
			>
				OAuth (Recommended)
			</button>
			<button
				type="button"
				class="flex-1 px-4 py-2 rounded-md text-sm font-medium transition-colors {!useOAuth
					? 'bg-background shadow-sm'
					: 'hover:bg-background/50'}"
				onclick={() => (useOAuth = false)}
			>
				App Password
			</button>
		</div>

		{#if useOAuth}
			<!-- OAuth Login Form -->
			<form onsubmit={(e) => { e.preventDefault(); handleOAuthLogin(); }} class="space-y-4">
				<div>
					<label for="oauth-handle" class="block text-sm font-medium mb-2">
						Bluesky Handle
					</label>
					<Input
						id="oauth-handle"
						type="text"
						bind:value={oauthHandle}
						placeholder="alice.bsky.social or @alice.bsky.social"
						required
						disabled={loading}
					/>
					<p class="text-xs text-muted-foreground mt-1">
						You'll be redirected to authorize access to your custom lexicons
					</p>
				</div>

				{#if error}
					<div class="p-3 bg-destructive/10 text-destructive rounded-md text-sm">
						{error}
					</div>
				{/if}

				<Button type="submit" class="w-full" disabled={loading}>
					{loading ? 'Redirecting...' : 'Login with OAuth'}
				</Button>

				<div class="p-3 bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 rounded-md text-xs">
					<strong>Why OAuth?</strong> OAuth allows you to grant specific permissions for custom
					lexicons (com.crown-shy.testing.*) while keeping your account secure.
				</div>
			</form>
		{:else}
			<!-- Password Login Form -->
			<form onsubmit={(e) => { e.preventDefault(); handleLogin(); }} class="space-y-4">
				<div>
					<label for="identifier" class="block text-sm font-medium mb-2">
						Handle or Email
					</label>
					<Input
						id="identifier"
						type="text"
						bind:value={identifier}
						placeholder="alice.bsky.social"
						required
						disabled={loading}
					/>
				</div>

				<div>
					<label for="password" class="block text-sm font-medium mb-2">
						App Password
					</label>
					<Input
						id="password"
						type="password"
						bind:value={password}
						placeholder="xxxx-xxxx-xxxx-xxxx"
						required
						disabled={loading}
					/>
					<p class="text-xs text-muted-foreground mt-1">
						Create an app password in your Bluesky settings
					</p>
				</div>

				{#if error}
					<div class="p-3 bg-destructive/10 text-destructive rounded-md text-sm">
						{error}
					</div>
				{/if}

				<Button type="submit" class="w-full" disabled={loading}>
					{loading ? 'Logging in...' : 'Login'}
				</Button>

				<div class="p-3 bg-yellow-50 dark:bg-yellow-950 text-yellow-700 dark:text-yellow-300 rounded-md text-xs">
					<strong>Note:</strong> App password login doesn't request specific lexicon scopes. Use
					OAuth for production.
				</div>
			</form>
		{/if}

		<p class="mt-4 text-sm text-muted-foreground text-center">
			Don't have a Bluesky account? <a
				href="https://bsky.app"
				target="_blank"
				class="text-primary hover:underline"
			>
				Create one here
			</a>
		</p>
	</Card>
</div>
