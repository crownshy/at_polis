<script lang="ts">
	import { user } from '$lib/stores/auth';
	import Button from '$lib/components/ui/Button.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import { goto } from '$app/navigation';

	let inviteLink = $state('');

	function handleJoinPoll() {
		if (!inviteLink) return;

		// Extract poll info from invite link
		// Format: /poll/{uri}/{cid}
		const match = inviteLink.match(/\/poll\/([^\/]+)\/([^\/]+)/);
		if (match) {
			const [, encodedUri, cid] = match;
			goto(`/poll/${encodedUri}/${cid}`);
		} else {
			alert('Invalid invite link');
		}
	}
</script>

<div class="space-y-8">
	<div class="text-center space-y-4">
		<h1 class="text-4xl font-bold">Welcome to ATProto Polis</h1>
		<p class="text-xl text-muted-foreground">
			A decentralized deliberation platform built on the AT Protocol
		</p>
	</div>

	{#if !$user}
		<Card class="p-6 max-w-md mx-auto">
			<h2 class="text-xl font-semibold mb-4">Get Started</h2>
			<p class="text-muted-foreground mb-4">
				Login with your Bluesky account to create polls and participate in discussions.
			</p>
			<Button onclick={() => goto('/login')} class="w-full">
				Login
			</Button>
		</Card>
	{:else}
		<div class="grid md:grid-cols-2 gap-6 max-w-4xl mx-auto">
			<Card class="p-6">
				<h2 class="text-xl font-semibold mb-4">Create a Poll</h2>
				<p class="text-muted-foreground mb-4">
					Start a new deliberation topic and invite others to participate.
				</p>
				<Button onclick={() => goto('/polls/create')} class="w-full">
					Create Poll
				</Button>
			</Card>

			<Card class="p-6">
				<h2 class="text-xl font-semibold mb-4">Join a Poll</h2>
				<p class="text-muted-foreground mb-4">
					Enter an invite link to join an existing poll.
				</p>
				<div class="space-y-2">
					<Input
						bind:value={inviteLink}
						placeholder="Paste invite link..."
					/>
					<Button onclick={handleJoinPoll} class="w-full" disabled={!inviteLink}>
						Join Poll
					</Button>
				</div>
			</Card>
		</div>

		<Card class="p-6 max-w-2xl mx-auto">
			<h2 class="text-xl font-semibold mb-4">How it Works</h2>
			<ol class="space-y-2 text-muted-foreground">
				<li>1. Create a poll and share the invite link</li>
				<li>2. Participants can add statements to the poll</li>
				<li>3. Everyone can vote on statements (agree, disagree, or pass)</li>
				<li>4. All data is stored in each user's Personal Data Server (PDS)</li>
				<li>5. No central server stores your data!</li>
			</ol>
		</Card>
	{/if}
</div>
