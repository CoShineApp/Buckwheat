<script lang="ts">
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLTableRowElement>> = $props();
</script>

<tr
	bind:this={ref}
	data-slot="table-row"
	class={cn(
		"odd:bg-muted/20 hover:[&,&>svelte-css-wrapper]:[&>th,td]:bg-primary/5 data-[state=selected]:bg-primary/10 border-b transition-colors",
		className
	)}
	{...restProps}
>
	{@render children?.()}
</tr>
