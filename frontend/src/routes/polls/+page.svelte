<script lang="ts">
	import { goto } from '$app/navigation';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'long',
			day: 'numeric'
		});
	}

	function viewPoll(uri: string, cid: string) {
		const encodedUri = encodeURIComponent(uri);
		goto(`/poll/${encodedUri}/${cid}`);
	}
</script>

<div class="container mx-auto px-4 py-8 max-w-4xl">
	<div class="mb-8">
		<h1 class="text-4xl font-bold mb-2">Available Polls</h1>
		<p class="text-muted-foreground">Browse and participate in ongoing deliberations</p>
	</div>

	{#if data.polls.length === 0}
		<Card>
			<div class="p-8 text-center">
				<p class="text-muted-foreground mb-4">No polls available yet</p>
				<Button onclick={() => goto('/polls/create')}>Create First Poll</Button>
			</div>
		</Card>
	{:else}
		<div class="space-y-4">
			{#each data.polls as poll}
				<Card>
					<div class="p-6">
						<div class="flex justify-between items-start mb-4">
							<div class="flex-1">
								<h2 class="text-2xl font-semibold mb-2">{poll.topic}</h2>
								{#if poll.description}
									<p class="text-muted-foreground mb-3">{poll.description}</p>
								{/if}
								<div class="flex gap-4 text-sm text-muted-foreground">
									<span>Created {formatDate(poll.createdAt)}</span>
									{#if poll.closedAt}
										<span class="text-destructive">• Closed {formatDate(poll.closedAt)}</span>
									{:else}
										<span class="text-green-600">• Active</span>
									{/if}
								</div>
							</div>
							{#if !poll.closedAt}
								<Button onclick={() => viewPoll(poll.uri, poll.cid)}>
									Participate
								</Button>
							{:else}
								<Button variant="outline" onclick={() => viewPoll(poll.uri, poll.cid)}>
									View Results
								</Button>
							{/if}
						</div>
					</div>
				</Card>
			{/each}
		</div>
	{/if}
</div>
