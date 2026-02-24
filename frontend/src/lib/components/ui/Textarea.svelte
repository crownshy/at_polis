<script lang="ts">
	interface Props {
		id?: string;
		value?: string;
		placeholder?: string;
		disabled?: boolean;
		required?: boolean;
		rows?: number;
		class?: string;
		oninput?: (e: Event) => void;
	}

	let {
		id,
		value = $bindable(''),
		placeholder = '',
		disabled = false,
		required = false,
		rows = 4,
		class: className = '',
		oninput
	}: Props = $props();

	const classes = $derived(
		`flex min-h-20 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 ${className}`
	);
</script>

<textarea {id} bind:value class={classes} {placeholder} {disabled} {required} {rows} {oninput}></textarea>
