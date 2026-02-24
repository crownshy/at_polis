<script lang="ts">
	interface Props {
		class?: string;
		children?: any;
	}

	let { class: className = '', children }: Props = $props();

	const classes = $derived(
		`rounded-lg border bg-card text-card-foreground shadow-sm ${className}`
	);
</script>

<div class={classes}>
	{@render children?.()}
</div>
