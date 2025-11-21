<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import Fuse from "fuse.js";
	import { FontAwesomeIcon } from '@fortawesome/svelte-fontawesome';
	import { faArrowLeft, faArrowRight, faPlus } from '@fortawesome/free-solid-svg-icons';
	import { apiService, type Song } from '$lib/api';
	const pageSize = 24;
	let search = "";
	let page = 0;
	let songs: Song[] = [];
	let fuse: Fuse<Song> | null = null;
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
			const response = await apiService.getSongs();
			if (response.success && response.data)
			{
				songs = response.data;
				fuse = new Fuse(songs, {
					keys: ["name", "artist_name"],
					threshold: 0.4,
				});
			}
		}
		catch (error)
		{
			console.error('Failed to load songs:', error);
		}
		finally
		{
			loading = false;
		}
	});
	$: filtered = search && fuse ? fuse.search(search).map(r => r.item) : songs;
	$: totalPages = Math.ceil(filtered.length / pageSize);
	$: pageItems = filtered.slice(page * pageSize, (page + 1) * pageSize);
	$: if (search) page = 0;
	function next() { if (page < totalPages - 1) page++; }
	function prev() { if (page > 0) page--; }
</script>
{#if loading}
	<div class="flex justify-center items-center p-8">
		<p>Loading songs...</p>
	</div>
{:else}
	<div class="flex justify-between items-center flex-wrap w-full">
		<h1 class="text-2xl">Total Registered Song {filtered.length}</h1>
		<input
			bind:value={search}
			placeholder="Search songs..."
			class="input input-bordered w-full max-w-sm"
		/>
	</div>
	<div class="grid gap-6 grid-cols-[repeat(auto-fill,minmax(theme(spacing.96),auto))] w-full mt-6">
		{#each pageItems as song}
			<div class="card card-hover aspect-square" style="max-width: 36rem;">
				<header class="card-header aspect-square w-full">
					<div class="aspect-square w-full rounded-md bg-black"></div>
				</header>
				<section class="min-h-0">
					<div class="flex flex-col justify-center items-center w-full text-center gap-1">
						<div class="font-semibold text-lg">{song.name}</div>
						<div class="text-sm text-gray-600">{song.artist_name}</div>
					</div>
				</section>
				<footer class="card-footer flex-shrink-0">
					<div class="flex justify-end">
						<button type="button" class="btn-icon variant-filled" aria-label="Add To Playlist">
							<FontAwesomeIcon icon={faPlus}/>
						</button>
					</div>
				</footer>
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