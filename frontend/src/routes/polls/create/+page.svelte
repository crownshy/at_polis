<script lang="ts">
	import { goto } from '$app/navigation';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Textarea from '$lib/components/ui/Textarea.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { user } from '$lib/stores/auth';
	import { createPoll } from '$lib/api/client';
	import { onMount } from 'svelte';

	let topic = $state('');
	let description = $state('');
	let loading = $state(false);
	let error = $state('');

	onMount(() => {
		if (!$user) {
			goto('/login');
		}
	});

	async function handleSubmit() {
		if (!$user) return;

		loading = true;
		error = '';

		try {
			const response = await createPoll({
				topic,
				description: description || undefined,
				identifier: $user.identifier,
				password: $user.password
			});

			if (response.success && response.uri && response.cid) {
				// Redirect to the poll page
				const encodedUri = encodeURIComponent(response.uri);
				const cleanCid = response.cid.replace(/^Cid\("(.+)"\)$/, '$1');
				goto(`/poll/${encodedUri}/${cleanCid}`);
			} else {
				error = response.message || 'Failed to create poll';
			}
		} catch (e) {
			error = 'Failed to connect to server';
		} finally {
			loading = false;
		}
	}
</script>

<div class="max-w-2xl mx-auto">
	<Card class="p-6">
		<h1 class="text-2xl font-bold mb-6">Create a New Poll</h1>

		<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="space-y-6">
			<div>
				<label for="topic" class="block text-sm font-medium mb-2">
					Topic <span class="text-destructive">*</span>
				</label>
				<Input
					id="topic"
					bind:value={topic}
					placeholder="What should we discuss?"
					required
					disabled={loading}
				/>
				<p class="text-xs text-muted-foreground mt-1">
					A clear, concise question or topic for deliberation (max 300 characters)
				</p>
			</div>

			<div>
				<label for="description" class="block text-sm font-medium mb-2">
					Description (optional)
				</label>
				<Textarea
					id="description"
					bind:value={description}
					placeholder="Provide more context about this poll..."
					rows={4}
					disabled={loading}
				/>
				<p class="text-xs text-muted-foreground mt-1">
					Additional details to help participants understand the topic (max 3000 characters)
				</p>
			</div>

			{#if error}
				<div class="p-3 bg-destructive/10 text-destructive rounded-md text-sm">
					{error}
				</div>
			{/if}

			<div class="flex gap-4">
				<Button type="button" variant="outline" onclick={() => goto('/')} disabled={loading}>
					Cancel
				</Button>
				<Button type="submit" disabled={loading || !topic} class="flex-1">
					{loading ? 'Creating...' : 'Create Poll'}
				</Button>
			</div>
		</form>
	</Card>
</div>
