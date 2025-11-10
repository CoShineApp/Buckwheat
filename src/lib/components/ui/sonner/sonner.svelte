<script lang="ts">
	import { Toaster as Sonner, type ToasterProps as SonnerProps } from "svelte-sonner";
	import { mode } from "mode-watcher";

	let { ...restProps }: SonnerProps = $props();
</script>

<Sonner
	theme={mode.current}
	class="toaster group"
	toastOptions={{
		classes: {
			error: "!bg-destructive !text-destructive-foreground !border-destructive",
			success: "!bg-green-600 !text-white !border-green-600 dark:!bg-green-600"
		}
	}}
	{...restProps}
/>
