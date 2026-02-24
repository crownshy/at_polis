<script lang="ts">
	interface Props {
		variant?: 'default' | 'destructive' | 'outline' | 'ghost';
		size?: 'default' | 'sm' | 'lg';
		disabled?: boolean;
		type?: 'button' | 'submit' | 'reset';
		onclick?: (e: MouseEvent) => void;
		class?: string;
		children?: any;
	}

	let {
		variant = 'default',
		size = 'default',
		disabled = false,
		type = 'button',
		onclick,
		class: className = '',
		children
	}: Props = $props();

	const variantClasses = {
		default: 'bg-primary text-primary-foreground hover:bg-primary/90',
		destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
		outline: 'border border-input bg-background hover:bg-accent hover:text-accent-foreground',
		ghost: 'hover:bg-accent hover:text-accent-foreground'
	};

	const sizeClasses = {
		default: 'h-10 px-4 py-2',
		sm: 'h-9 px-3',
		lg: 'h-11 px-8'
	};

	const classes = $derived(
		`inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 ${variantClasses[variant]} ${sizeClasses[size]} ${className}`
	);
</script>

<button {type} class={classes} {disabled} {onclick}>
	{@render children?.()}
</button>
