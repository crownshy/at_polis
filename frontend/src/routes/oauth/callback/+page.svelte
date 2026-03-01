<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import Card from '$lib/components/ui/Card.svelte';

	let status = $state<'processing' | 'success' | 'error'>('processing');
	let message = $state('Processing OAuth callback...');

	onMount(async () => {
		// The backend automatically handles the token exchange
		// when this URL is accessed with code and state parameters

		try {
			// Check if we're authenticated
			const response = await fetch('http://localhost:3000/me', {
				credentials: 'include', // Important for cookies
			});

			const data = await response.json();

			if (data.authenticated && data.session) {
				status = 'success';
				message = 'Authentication successful! Redirecting...';

				// Store session info in local storage
				localStorage.setItem('user', JSON.stringify(data.session));

				// Redirect to home page after a short delay
				setTimeout(() => {
					goto('/');
				}, 1500);
			} else {
				status = 'error';
				message = 'Authentication failed. Please try again.';
			}
		} catch (e) {
			status = 'error';
			message = 'Failed to verify authentication. Please try logging in again.';
		}
	});
</script>

<div class="flex items-center justify-center min-h-[60vh]">
	<Card class="w-full max-w-md p-6">
		<h1 class="text-2xl font-bold mb-6">OAuth Callback</h1>

		<div class="space-y-4">
			{#if status === 'processing'}
				<div class="flex items-center justify-center p-8">
					<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
				</div>
			{:else if status === 'success'}
				<div class="p-4 bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300 rounded-md text-center">
					<svg
						class="w-12 h-12 mx-auto mb-2"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M5 13l4 4L19 7"
						></path>
					</svg>
					<p class="font-medium">{message}</p>
				</div>
			{:else}
				<div class="p-4 bg-destructive/10 text-destructive rounded-md text-center">
					<svg
						class="w-12 h-12 mx-auto mb-2"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18L18 6M6 6l12 12"
						></path>
					</svg>
					<p class="font-medium mb-2">{message}</p>
					<a href="/login" class="text-sm underline hover:no-underline">
						Return to login
					</a>
				</div>
			{/if}

			<div class="text-xs text-muted-foreground text-center">
				<p>Processing OAuth authentication with custom lexicon scopes:</p>
				<ul class="mt-2 space-y-1 text-left max-w-xs mx-auto">
					<li>• com.crown-shy.testing.poll</li>
					<li>• com.crown-shy.testing.statement</li>
					<li>• com.crown-shy.testing.vote</li>
				</ul>
			</div>
		</div>
	</Card>
</div>
