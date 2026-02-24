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
		</form>

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
