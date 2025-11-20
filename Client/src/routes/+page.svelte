<script lang="ts">
	import { AppRail, AppRailAnchor, AppRailTile } from '@skeletonlabs/skeleton';
	import { FontAwesomeIcon } from '@fortawesome/svelte-fontawesome';
	import { faListUl, faMusic, faPalette, faUser } from '@fortawesome/free-solid-svg-icons';
	import SongsList from './components/songs-list.svelte';
	import ArtistList from './components/artist-list.svelte';

	let currentTile: number = 0;
</script>

<div class="h-screen w-screen flex">
	<!-- LEFT SIDE: Rail -->
	<AppRail height="h-full" class="shrink-0">
		<AppRailTile bind:group={currentTile} name="Songs" value={0}>
			<svelte:fragment slot="lead"><FontAwesomeIcon icon={faMusic} size="2x" /></svelte:fragment>
			<span>Songs</span>
		</AppRailTile>
		<AppRailTile bind:group={currentTile} name="Artists" value={1}>
			<svelte:fragment slot="lead"><FontAwesomeIcon icon={faPalette} size="2x" /></svelte:fragment>
			<span>Artists</span>
		</AppRailTile>
		<AppRailTile bind:group={currentTile} name="Playlists" value={2}>
			<svelte:fragment slot="lead"><FontAwesomeIcon icon={faListUl} size="2x" /></svelte:fragment>
			<span>Playlists</span>
		</AppRailTile>

		<svelte:fragment slot="trail">
			<AppRailAnchor href="/" target="_blank" title="Account">
				<FontAwesomeIcon icon={faUser} size="3x" />
			</AppRailAnchor>
		</svelte:fragment>
	</AppRail>
	<div class="flex-1 overflow-auto p-4">
		{#if (currentTile === 0)}
			<SongsList />
		{/if}
		{#if (currentTile === 1)}
			<ArtistList />
		{/if}
	</div>
</div>
