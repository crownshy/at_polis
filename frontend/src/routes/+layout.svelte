<script lang="ts">
	import '../app.css';
	import { user } from '$lib/stores/auth';

	let { children } = $props();
</script>

<div class="min-h-screen bg-background">
	<nav class="border-b">
		<div class="container mx-auto px-4 py-4 flex items-center justify-between">
			<a href="/" class="text-2xl font-bold">ATProto Polis</a>
			<div class="flex gap-4">
				{#if $user}
					<span class="text-muted-foreground">@{$user.handle}</span>
					<a href="/polls/create" class="hover:text-primary">Create Poll</a>
					<a href="/polls" class="hover:text-primary">My Polls</a>
					<button
						onclick={() => {
							user.set(null);
							window.location.href = '/login';
						}}
						class="hover:text-destructive"
					>
						Logout
					</button>
				{:else}
					<a href="/login" class="hover:text-primary">Login</a>
				{/if}
			</div>
		</div>
	</nav>

	<main class="container mx-auto px-4 py-8">
		{@render children()}
	</main>
</div>
