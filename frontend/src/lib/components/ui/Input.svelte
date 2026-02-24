<script lang="ts">
	interface Props {
		id?: string;
		type?: string;
		value?: string;
		placeholder?: string;
		disabled?: boolean;
		required?: boolean;
		readonly?: boolean;
		class?: string;
		oninput?: (e: Event) => void;
	}

	let {
		id,
		type = 'text',
		value = $bindable(''),
		placeholder = '',
		disabled = false,
		required = false,
		readonly = false,
		class: className = '',
		oninput
	}: Props = $props();

	const classes = $derived(
		`flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 ${className}`
	);
</script>

<input {id} {type} bind:value class={classes} {placeholder} {disabled} {required} readonly={readonly} {oninput} />
