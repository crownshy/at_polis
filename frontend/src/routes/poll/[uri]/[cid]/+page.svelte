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

	// Mock data for statements (in production, this would be fetched from the backend)
	let statements = $state<
		Array<{
			uri: string;
			cid: string;
			text: string;
			author: string;
			votes?: { agree: number; disagree: number; pass: number };
			userVote?: "agree" | "disagree" | "pass";
		}>
	>([]);

	let { data } = $props();
	console.log("data ", data);

	onMount(() => {
		if (!data.authenticated) {
			goto("/login");
			return;
		}

		// Generate invite link
		const encodedUri = encodeURIComponent(pollUri);
		inviteLink = `${window.location.origin}/poll/${encodedUri}/${pollCid}`;
	});

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
				// Add the new statement to the local list
				const cleanCid = response.cid.replace(/^Cid\("(.+)"\)$/, "$1");
				statements = [
					...statements,
					{
						uri: response.uri,
						cid: cleanCid,
						text: newStatement,
						author: data.did,
						votes: { agree: 0, disagree: 0, pass: 0 },
					},
				];
				newStatement = "";
			} else {
				error = response.message || "Failed to create statement";
			}
		} catch (e) {
			error = "Failed to connect to server";
		} finally {
			loading = false;
		}
	}

	async function handleVote(
		statement: (typeof statements)[0],
		value: "agree" | "disagree" | "pass",
	) {
		if (!data.authenticated) return;

		try {
			const response = await createVote({
				value,
				statement_uri: statement.uri,
				statement_cid: statement.cid,
				poll_uri: pollUri,
				poll_cid: pollCid,
			});

			if (response.success) {
				// Update the local vote count and user vote
				const index = statements.findIndex((s) => s.uri === statement.uri);
				if (index !== -1) {
					const updated = [...statements];
					if (updated[index].votes) {
						// Remove previous vote if exists
						if (updated[index].userVote) {
							updated[index].votes![updated[index].userVote!]--;
						}
						// Add new vote
						updated[index].votes![value]++;
						updated[index].userVote = value;
					}
					statements = updated;
				}
			}
		} catch (e) {
			console.error("Vote failed:", e);
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
		<h1 class="text-3xl font-bold mb-2">Poll Details</h1>
		<p class="text-muted-foreground mb-4">
			Poll URI: <code class="text-xs">{pollUri}</code>
		</p>

		<div class="flex gap-2">
			<Input value={inviteLink} readonly class="flex-1" />
			<Button onclick={copyInviteLink}>
				{copied ? "Copied!" : "Copy Link"}
			</Button>
		</div>
		<p class="text-xs text-muted-foreground mt-2">
			Share this link with others to invite them to participate
		</p>
	</Card>

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

			{#if error}
				<div class="p-3 bg-destructive/10 text-destructive rounded-md text-sm">
					{error}
				</div>
			{/if}

			<Button type="submit" disabled={loading || !newStatement.trim()}>
				{loading ? "Adding..." : "Add Statement"}
			</Button>
		</form>
	</Card>

	<!-- Statements List -->
	<div class="space-y-4">
		<h2 class="text-2xl font-semibold">Statements</h2>

		{#if statements.length === 0}
			<Card class="p-6 text-center text-muted-foreground">
				No statements yet. Be the first to add one!
			</Card>
		{:else}
			{#each statements as statement (statement.uri)}
				<Card class="p-6">
					<p class="text-lg mb-2">{statement.text}</p>
					<p class="text-sm text-muted-foreground mb-4">
						by @{statement.author}
					</p>

					<div class="flex items-center gap-4">
						<Button
							size="sm"
							variant={statement.userVote === "agree" ? "default" : "outline"}
							onclick={() => handleVote(statement, "agree")}
						>
							👍 Agree {statement.votes?.agree || 0}
						</Button>
						<Button
							size="sm"
							variant={statement.userVote === "disagree"
								? "default"
								: "outline"}
							onclick={() => handleVote(statement, "disagree")}
						>
							👎 Disagree {statement.votes?.disagree || 0}
						</Button>
						<Button
							size="sm"
							variant={statement.userVote === "pass" ? "default" : "outline"}
							onclick={() => handleVote(statement, "pass")}
						>
							🤷 Pass {statement.votes?.pass || 0}
						</Button>
					</div>
				</Card>
			{/each}
		{/if}
	</div>
</div>
