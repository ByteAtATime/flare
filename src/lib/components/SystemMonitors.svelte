<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	interface CpuInfo {
		usage_percent: number;
		cores: Array<{ index: number; usage_percent: number }>;
	}

	interface MemoryInfo {
		total_bytes: number;
		used_bytes: number;
		available_bytes: number;
		usage_percent: number;
	}

	interface DiskInfo {
		name: string;
		mount_point: string;
		total_bytes: number;
		used_bytes: number;
		available_bytes: number;
		usage_percent: number;
		file_system: string;
	}

	interface NetworkInfo {
		interface: string;
		bytes_sent: number;
		bytes_received: number;
		packets_sent: number;
		packets_received: number;
	}

	interface BatteryInfo {
		percentage: number;
		is_charging: boolean;
		is_present: boolean;
		time_remaining_minutes: number | null;
	}

	let cpu: CpuInfo | null = $state(null);
	let memory: MemoryInfo | null = $state(null);
	let disks: DiskInfo[] = $state([]);
	let network: NetworkInfo[] = $state([]);
	let battery: BatteryInfo | null = $state(null);
	let loading = $state(true);

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
	}

	async function refreshData() {
		try {
			loading = true;
			const [cpuData, memData, diskData, netData, batData] = await Promise.all([
				invoke<CpuInfo>('monitor_get_cpu'),
				invoke<MemoryInfo>('monitor_get_memory'),
				invoke<DiskInfo[]>('monitor_get_disks'),
				invoke<NetworkInfo[]>('monitor_get_network'),
				invoke<BatteryInfo | null>('monitor_get_battery')
			]);

			cpu = cpuData;
			memory = memData;
			disks = diskData;
			network = netData;
			battery = batData;
		} catch (error) {
			console.error('Failed to fetch system info:', error);
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		refreshData();
		// Refresh every 2 seconds
		const interval = setInterval(refreshData, 2000);
		return () => clearInterval(interval);
	});
</script>

<div class="p-6 space-y-6">
	<div class="flex items-center justify-between">
		<h2 class="text-2xl font-bold">System Monitors</h2>
		<button
			onclick={refreshData}
			class="rounded-md bg-primary px-3 py-1.5 text-sm text-primary-foreground hover:bg-primary/90"
		>
			Refresh
		</button>
	</div>

	{#if loading && !cpu}
		<div class="text-muted-foreground">Loading system information...</div>
	{:else}
		<!-- CPU Info -->
		{#if cpu}
			<div class="rounded-lg border p-4">
				<h3 class="mb-3 text-lg font-semibold">CPU Usage</h3>
				<div class="mb-2">
					<div class="flex items-center justify-between text-sm">
						<span>Overall</span>
						<span class="font-mono">{cpu.usage_percent.toFixed(1)}%</span>
					</div>
					<div class="mt-1 h-2 w-full overflow-hidden rounded-full bg-secondary">
						<div
							class="h-full bg-primary transition-all"
							style="width: {cpu.usage_percent}%"
						></div>
					</div>
				</div>
				<div class="mt-4 grid grid-cols-2 gap-2 md:grid-cols-4">
					{#each cpu.cores as core}
						<div class="text-xs">
							<div class="flex justify-between">
								<span class="text-muted-foreground">Core {core.index}</span>
								<span class="font-mono">{core.usage_percent.toFixed(0)}%</span>
							</div>
							<div class="mt-1 h-1 w-full overflow-hidden rounded-full bg-secondary">
								<div
									class="h-full bg-primary/70 transition-all"
									style="width: {core.usage_percent}%"
								></div>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Memory Info -->
		{#if memory}
			<div class="rounded-lg border p-4">
				<h3 class="mb-3 text-lg font-semibold">Memory Usage</h3>
				<div class="flex items-center justify-between text-sm">
					<span>{formatBytes(memory.used_bytes)} / {formatBytes(memory.total_bytes)}</span>
					<span class="font-mono">{memory.usage_percent.toFixed(1)}%</span>
				</div>
				<div class="mt-2 h-2 w-full overflow-hidden rounded-full bg-secondary">
					<div
						class="h-full bg-primary transition-all"
						style="width: {memory.usage_percent}%"
					></div>
				</div>
				<div class="mt-2 text-xs text-muted-foreground">
					Available: {formatBytes(memory.available_bytes)}
				</div>
			</div>
		{/if}

		<!-- Disk Info -->
		{#if disks.length > 0}
			<div class="rounded-lg border p-4">
				<h3 class="mb-3 text-lg font-semibold">Disk Usage</h3>
				<div class="space-y-3">
					{#each disks as disk}
						<div>
							<div class="flex items-center justify-between text-sm">
								<div>
									<span class="font-medium">{disk.mount_point}</span>
									<span class="ml-2 text-xs text-muted-foreground">({disk.file_system})</span>
								</div>
								<span class="font-mono">{disk.usage_percent.toFixed(1)}%</span>
							</div>
							<div class="mt-1 h-2 w-full overflow-hidden rounded-full bg-secondary">
								<div
									class="h-full bg-primary transition-all"
									style="width: {disk.usage_percent}%"
								></div>
							</div>
							<div class="mt-1 text-xs text-muted-foreground">
								{formatBytes(disk.used_bytes)} / {formatBytes(disk.total_bytes)} ({formatBytes(
									disk.available_bytes
								)} free)
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Network Info -->
		{#if network.length > 0}
			<div class="rounded-lg border p-4">
				<h3 class="mb-3 text-lg font-semibold">Network Interfaces</h3>
				<div class="space-y-2">
					{#each network as net}
						<div class="text-sm">
							<div class="font-medium">{net.interface}</div>
							<div class="mt-1 grid grid-cols-2 gap-2 text-xs text-muted-foreground">
								<div>↓ Received: {formatBytes(net.bytes_received)}</div>
								<div>↑ Sent: {formatBytes(net.bytes_sent)}</div>
								<div>Packets RX: {net.packets_received.toLocaleString()}</div>
								<div>Packets TX: {net.packets_sent.toLocaleString()}</div>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Battery Info -->
		{#if battery}
			<div class="rounded-lg border p-4">
				<h3 class="mb-3 text-lg font-semibold">Battery</h3>
				<div class="flex items-center justify-between text-sm">
					<div class="flex items-center gap-2">
						<span>{battery.percentage.toFixed(0)}%</span>
						{#if battery.is_charging}
							<span class="text-xs text-green-600 dark:text-green-400">⚡ Charging</span>
						{:else}
							<span class="text-xs text-muted-foreground">Discharging</span>
						{/if}
					</div>
					{#if battery.time_remaining_minutes}
						<span class="text-xs text-muted-foreground">
							{Math.floor(battery.time_remaining_minutes / 60)}h {battery.time_remaining_minutes %
								60}m remaining
						</span>
					{/if}
				</div>
				<div class="mt-2 h-2 w-full overflow-hidden rounded-full bg-secondary">
					<div
						class="h-full transition-all"
						class:bg-green-500={battery.percentage > 50}
						class:bg-yellow-500={battery.percentage > 20 && battery.percentage <= 50}
						class:bg-red-500={battery.percentage <= 20}
						style="width: {battery.percentage}%"
					></div>
				</div>
			</div>
		{/if}
	{/if}
</div>
