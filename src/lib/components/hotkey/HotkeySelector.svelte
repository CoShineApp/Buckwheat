<script lang="ts">
	import { InputGroup, InputGroupInput, InputGroupButton } from "$lib/components/ui/input-group";
	import { X } from "@lucide/svelte";

	let {
		value = $bindable(""),
		placeholder = "Press a key...",
		onchange,
	}: {
		value?: string;
		placeholder?: string;
		onchange?: (value: string) => void;
	} = $props();

	let isRecording = $state(false);
	let inputRef: HTMLInputElement | null = $state(null);

	function formatHotkey(event: KeyboardEvent): string {
		const parts: string[] = [];

		if (event.ctrlKey || event.metaKey) parts.push(event.metaKey ? "Cmd" : "Ctrl");
		if (event.altKey) parts.push("Alt");
		if (event.shiftKey) parts.push("Shift");

		// Don't include modifier keys themselves
		const key = event.key;
		if (!["Control", "Alt", "Shift", "Meta"].includes(key)) {
			// Capitalize first letter and handle special keys
			const formattedKey = key.length === 1 ? key.toUpperCase() : key;
			parts.push(formattedKey);
		}

		return parts.join("+");
	}

	function startRecording(): void {
		isRecording = true;
		if (inputRef) {
			inputRef.focus();
		}
	}

	function stopRecording(): void {
		isRecording = false;
		if (inputRef) {
			inputRef.blur();
		}
	}

	function handleKeyDown(event: KeyboardEvent): void {
		if (!isRecording) return;

		event.preventDefault();
		event.stopPropagation();

		// Allow Escape to cancel
		if (event.key === "Escape") {
			stopRecording();
			return;
		}

		// Only capture when a non-modifier key is pressed
		// This ensures we get the full combination (e.g., Ctrl+A, not just Ctrl)
		const isModifierKey = ["Control", "Alt", "Shift", "Meta"].includes(event.key);
		if (isModifierKey) {
			return; // Wait for the actual key press
		}

		// Format and set the hotkey
		const hotkey = formatHotkey(event);
		if (hotkey) {
			value = hotkey;
			onchange?.(hotkey);
			stopRecording();
		}
	}

	function clearHotkey(): void {
		value = "";
		onchange?.("");
	}
</script>

<InputGroup>
	<InputGroupInput
		bind:ref={inputRef}
		type="text"
		readonly
		{placeholder}
		{value}
		onfocus={startRecording}
		onblur={stopRecording}
		onkeydown={handleKeyDown}
		class={isRecording ? "ring-2 ring-primary" : ""}
	/>
	{#if value}
		<InputGroupButton onclick={clearHotkey}>
			<X class="size-4" />
		</InputGroupButton>
	{/if}
</InputGroup>

