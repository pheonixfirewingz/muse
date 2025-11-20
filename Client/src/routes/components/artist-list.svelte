<script lang="ts">
	import Fuse from "fuse.js";
	import { faArrowLeft, faArrowRight } from '@fortawesome/free-solid-svg-icons';
	import { FontAwesomeIcon } from '@fortawesome/svelte-fontawesome';
	const pageSize = 24;

	const test_data: number[] = [
		1, 2, 3, 4, 6, 7, 8,
		9,10,11,12,13,14,15,
		16,17,18,19,20,21,22,
		23,24,25,26,27,28,29
	];

	let search = "";
	let page = 0;

	const fuse = new Fuse(test_data.map(v => ({ value: v })), {
		keys: ["value"],
		threshold: 0.4,
	});

	$: filtered = search ? fuse.search(search).map(r => r.item.value) : test_data;
	$: totalPages = Math.ceil(filtered.length / pageSize);
	$: pageItems = filtered.slice(page * pageSize, (page + 1) * pageSize);
	$: if (search) page = 0;

	function next() { if (page < totalPages - 1) page++; }
	function prev() { if (page > 0) page--; }
</script>

<div class="flex justify-between items-center flex-wrap w-full">
	<h1 class="text-2xl">Total Registered Artists {filtered.length}</h1>

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
			<section class="min-h-0 pb-4">
				<div class="flex justify-center flex-wrap w-full text-lg">
					(content) {song}
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
