<script lang="ts">
	import { page } from "$app/stores";
	import { onMount } from "svelte";
	import Button from "$lib/components/ui/Button.svelte";
	import Input from "$lib/components/ui/Input.svelte";
	import Textarea from "$lib/components/ui/Textarea.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import { createStatement, createVote } from "$lib/api/client";
	import { goto } from "$app/navigation";

	const pollUri = $derived(decodeURIComponent($page.params.uri || ""));
	const pollCid = $derived($page.params.cid || "");

	let newStatement = $state("");
	let loading = $state(false);
	let error = $state("");
	let inviteLink = $state("");
	let copied = $state(false);
	let votingLoading = $state(false);

	interface Statement {
		uri: string;
		cid: string;
		did: string;
		text: string;
		createdAt: string;
		poll: {
			uri: string;
			cid: string;
		};
	}

	let currentStatement = $state<Statement | null>(null);
	let noMoreStatements = $state(false);

	let { data } = $props();

	onMount(() => {
		if (!data.authenticated) {
			goto("/login");
			return;
		}

		// Generate invite link
		const encodedUri = encodeURIComponent(pollUri);
		inviteLink = `${window.location.origin}/poll/${encodedUri}/${pollCid}`;

		// Fetch first statement
		fetchNextStatement();
	});

	async function fetchNextStatement() {
		try {
			const response = await fetch(
				`/api/polis/${encodeURIComponent(pollUri)}/next_statement`,
			);
			if (!response.ok) {
				throw new Error("Failed to fetch next statement");
			}

			const statement: Statement | null = await response.json();

			if (statement === null) {
				noMoreStatements = true;
				currentStatement = null;
			} else {
				noMoreStatements = false;
				currentStatement = statement;
			}
		} catch (e) {
			console.error("Failed to fetch next statement:", e);
			error = "Failed to load statement";
		}
	}

	async function handleAddStatement() {
		if (!data.authenticated || !newStatement.trim()) return;

		loading = true;
		error = "";

		try {
			const response = await createStatement({
				text: newStatement,
				poll_uri: pollUri,
				poll_cid: pollCid,
			});

			if (response.success && response.uri && response.cid) {
				newStatement = "";
				// Fetch next statement in case the one we just added becomes votable
				await fetchNextStatement();
			} else {
				error = response.message || "Failed to create statement";
			}
		} catch (e) {
			error = "Failed to connect to server";
		} finally {
			loading = false;
		}
	}

	async function handleVote(value: "agree" | "disagree" | "pass") {
		console.log("handling vote");
		if (!data.authenticated || !currentStatement) return;
		console.log("did not pass");

		votingLoading = true;
		error = "";

		try {
			const response = await createVote({
				value,
				statement_uri: currentStatement.uri,
				statement_cid: currentStatement.cid,
				poll_uri: pollUri,
				poll_cid: pollCid,
			});

			if (response.success) {
				// Fetch the next statement after voting
				await fetchNextStatement();
			} else {
				error = response.message || "Failed to record vote";
			}
		} catch (e) {
			console.error("Vote failed:", e);
			error = "Failed to submit vote";
		} finally {
			votingLoading = false;
		}
	}

	function copyInviteLink() {
		navigator.clipboard.writeText(inviteLink);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}
</script>

<div class="max-w-4xl mx-auto space-y-6">
	<!-- Poll Header -->
	<Card class="p-6">
		{#if data.poll}
			<h1 class="text-3xl font-bold mb-2">{data.poll.topic}</h1>
			{#if data.poll.description}
				<p class="text-lg text-muted-foreground mb-4">{data.poll.description}</p>
			{/if}
		{:else}
			<h1 class="text-3xl font-bold mb-2">Poll</h1>
			<p class="text-muted-foreground mb-4">
				<code class="text-xs">{pollUri}</code>
			</p>
		{/if}

		<div class="flex gap-2 mb-2">
			<Input value={inviteLink} readonly class="flex-1" />
			<Button onclick={copyInviteLink}>
				{copied ? "Copied!" : "Copy Link"}
			</Button>
		</div>
		<p class="text-xs text-muted-foreground">
			Share this link with others to invite them to participate
		</p>
	</Card>

	<!-- Voting Section -->
	{#if currentStatement}
		<Card class="p-8">
			<h2 class="text-xl font-semibold mb-4">
				How do you feel about this statement?
			</h2>

			<div class="bg-muted p-6 rounded-lg mb-6">
				<p class="text-lg">{currentStatement.text}</p>
				<p class="text-sm text-muted-foreground mt-2">
					by @{currentStatement.did}
				</p>
			</div>

			{#if error}
				<div
					class="p-3 bg-destructive/10 text-destructive rounded-md text-sm mb-4"
				>
					{error}
				</div>
			{/if}

			<div class="flex gap-3 justify-center">
				<Button
					size="lg"
					variant="outline"
					onclick={() => handleVote("disagree")}
					disabled={votingLoading}
					class="flex-1"
				>
					👎 Disagree
				</Button>
				<Button
					size="lg"
					variant="outline"
					onclick={() => handleVote("pass")}
					disabled={votingLoading}
					class="flex-1"
				>
					🤷 Pass
				</Button>
				<Button
					size="lg"
					variant="outline"
					onclick={() => handleVote("agree")}
					disabled={votingLoading}
					class="flex-1"
				>
					👍 Agree
				</Button>
			</div>
		</Card>
	{:else if noMoreStatements}
		<Card class="p-8 text-center">
			<h2 class="text-2xl font-semibold mb-2">All caught up! 🎉</h2>
			<p class="text-muted-foreground mb-4">
				You've voted on all available statements. Add a new statement below to
				continue the conversation.
			</p>
		</Card>
	{:else}
		<Card class="p-8 text-center">
			<p class="text-muted-foreground">Loading statement...</p>
		</Card>
	{/if}

	<!-- Add Statement -->
	<Card class="p-6">
		<h2 class="text-xl font-semibold mb-4">Add a Statement</h2>
		<form
			onsubmit={(e) => {
				e.preventDefault();
				handleAddStatement();
			}}
			class="space-y-4"
		>
			<Textarea
				bind:value={newStatement}
				placeholder="Share your perspective on this topic..."
				rows={3}
				disabled={loading}
			/>

			<Button type="submit" disabled={loading || !newStatement.trim()}>
				{loading ? "Adding..." : "Add Statement"}
			</Button>
		</form>
	</Card>
</div>
