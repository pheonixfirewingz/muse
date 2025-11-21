<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import Fuse from "fuse.js";
	import { FontAwesomeIcon } from '@fortawesome/svelte-fontawesome';
	import { faArrowLeft, faArrowRight } from '@fortawesome/free-solid-svg-icons';
	import { apiService, type Artist } from '$lib/api';
	const pageSize = 24;
	let search = "";
	let page = 0;
	let Artists: Artist[] = [];
	let fuse: Fuse<Artist> | null = null;
	let loading = true;
	onMount(async () =>
	{
		if (!apiService.isAuthenticated())
		{
			goto('/login');
			return;
		}

		try
		{
			const response = await apiService.getArtists();
			if (response.success && response.data)
			{
				Artists = response.data;
				fuse = new Fuse(Artists, {
					keys: ["name"],
					threshold: 0.4,
				});
			}
		}
		catch (error)
		{
			console.error('Failed to load Artists:', error);
		}
		finally
		{
			loading = false;
		}
	});

	$: filtered = search && fuse ? fuse.search(search).map(r => r.item) : Artists;
	$: totalPages = Math.ceil(filtered.length / pageSize);
	$: pageItems = filtered.slice(page * pageSize, (page + 1) * pageSize);
	$: if (search) page = 0;

	function next() { if (page < totalPages - 1) page++; }
	function prev() { if (page > 0) page--; }
</script>
{#if loading}
	<div class="flex justify-center items-center p-8">
		<p>Loading Artists...</p>
	</div>
{:else}
	<div class="flex justify-between items-center flex-wrap w-full">
		<h1 class="text-2xl">Total Registered Artists {filtered.length}</h1>

		<input
			bind:value={search}
			placeholder="Search Artists..."
			class="input input-bordered w-full max-w-sm"
		/>
	</div>

	<div class="grid gap-6 grid-cols-[repeat(auto-fill,minmax(theme(spacing.96),auto))] w-full mt-6">
		{#each pageItems as artist}
			<div class="card card-hover aspect-square" style="max-width: 36rem;">
				<header class="card-header aspect-square w-full">
					<div class="aspect-square w-full rounded-md bg-black"></div>
				</header>
				<section class="min-h-0 p-4">
					<div class="flex flex-col justify-center items-center w-full text-center gap-1">
						<div class="font-semibold text-lg">{artist.name}</div>
					</div>
				</section>
			</div>
		{/each}
	</div>
	{#if filtered.length > pageSize}
		<div class="flex justify-center items-center gap-6 mt-8">
			<button class="btn-icon variant-filled" aria-label="Prev Page"
							on:click={prev} disabled={page === 0}>
				<FontAwesomeIcon icon={faArrowLeft}/>
			</button>
			<div>Page {page + 1} / {totalPages}</div>
			<button class="btn-icon variant-filled" aria-label="Next Page"
							on:click={next} disabled={page === totalPages - 1}>
				<FontAwesomeIcon icon={faArrowRight}/>
			</button>
		</div>
	{/if}
{/if}