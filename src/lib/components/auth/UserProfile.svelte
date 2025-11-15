<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { auth } from '$lib/stores/auth.svelte';
	import { toast } from 'svelte-sonner';
	import { LogOut, User } from '@lucide/svelte';
	import StorageUsageBar from '$lib/components/cloud/StorageUsageBar.svelte';

	async function handleLogout() {
		const result = await auth.signOut();
		if (result.success) {
			toast.success('Logged out successfully');
		} else {
			toast.error(result.error || 'Failed to log out');
		}
	}
</script>

{#if auth.isAuthenticated && auth.user}
	<Card>
		<CardHeader>
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<User class="size-5" />
					<CardTitle>Account</CardTitle>
				</div>
				<Button variant="ghost" size="sm" onclick={handleLogout}>
					<LogOut class="size-4 mr-2" />
					Log Out
				</Button>
			</div>
			<CardDescription>{auth.user.email}</CardDescription>
		</CardHeader>
	</Card>

	<StorageUsageBar />
{/if}

