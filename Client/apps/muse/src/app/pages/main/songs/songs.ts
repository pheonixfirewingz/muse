import { Component, inject, OnInit, OnDestroy } from '@angular/core';
import { MatCard, MatCardActions, MatCardContent } from '@angular/material/card';
import { MatButton } from '@angular/material/button';
import { FaIconComponent } from '@fortawesome/angular-fontawesome';
import { faArrowLeft, faArrowRight, faPlus } from '@fortawesome/free-solid-svg-icons';
import { MatDialog } from '@angular/material/dialog';
import { AddToPlaylistPopup } from '../../../component/add-to-playlist-popup/add-to-playlist-popup';
import { Song } from '../../../data/song';
import { environment } from '../../../../environments/environment';
import { fetchWithAuth } from '../../../app';
import { Router } from '@angular/router';
import { MetaCacheService } from '../../../services/meta-cache.service';
import { MusicPlayerService } from '../../../services/music-player.service';
import { SongSearchResult } from '../../../services/music-player.service';
import { ImageLoaderService } from '../../../services/image-loader.service';

@Component({
  selector: 'app-songs',
  standalone: true,
  imports: [
    MatCard,
    MatCardContent,
    MatCardActions,
    FaIconComponent,
    MatButton,
  ],
  templateUrl: './songs.html',
  styleUrls: ['./songs.scss', '../../shared/card.scss'],
})
export class Songs implements OnInit, OnDestroy {
  private max_count: number = 0;
  protected spanStart: number = 0;
  protected spanEnd: number = 36;
  protected songs_data: Song[] = [];
  protected readonly faPlus = faPlus;
  protected readonly faArrowRight = faArrowRight;
  protected readonly faArrowLeft = faArrowLeft;
  private readonly playlist_dialog: MatDialog = inject(MatDialog);
  private readonly router = inject(Router);
  protected songCoverUrls = new Map<string, string>();
  protected songCoverLoading = new Map<string, boolean>();
  private objectUrls: string[] = [];

  private musicPlayerService: MusicPlayerService = inject(MusicPlayerService);

  constructor(private imageLoaderService: ImageLoaderService) {
    // No per-component worker logic needed
  }

  getCoverUrl(song: Song): string {
    const key = `${song.artist}___${song.name}`;
    return this.songCoverUrls.get(key) ?? 'place_holder.webp';
  }

  isCoverLoading(song: Song): boolean {
    const key = `${song.artist}___${song.name}`;
    return this.songCoverLoading.get(key) ?? false;
  }

  async ngOnInit(): Promise<void> {
    try {
      let total = await MetaCacheService.getTotal('songs');
      if (total !== null) {
        this.max_count = total + 36;
      } else {
        const token = localStorage.getItem('authToken');
        const totalResponse = await fetchWithAuth(`${environment.apiUrl}/api/songs/total`, { headers: { Authorization: `Bearer ${token}` } }, this.router);
        const totalData = await totalResponse.json();
        this.max_count = totalData.data.total + 36;
        await MetaCacheService.setTotal('songs', totalData.data.total);
      }
      await this.getSongs();
    } catch (error) {
      console.error(error);
    }
  }

  async getSongs() {
    this.songs_data = [];
    // Clear image maps to avoid stale state
    this.songCoverUrls.clear();
    this.songCoverLoading.clear();
    const key = `songs_${this.spanStart}_${this.spanEnd}`;
    // Try cache first
    const cached = await MetaCacheService.getSongs(key);
    if (cached) {
      for (let song of cached) {
        this.songs_data.push(new Song(song.name, song.artist_name));
      }
      await this.preloadSongCovers();
      return;
    }
    // If not cached, fetch from server
    const token = localStorage.getItem('authToken');
    const url = new URL(`${environment.apiUrl}/api/songs`);
    url.searchParams.append('index_start', this.spanStart.toString());
    url.searchParams.append('index_end', this.spanEnd.toString());
    const songsResponse = await fetchWithAuth(url.toString(), { headers: { Authorization: `Bearer ${token}` } }, this.router);
    const songsData = await songsResponse.json();
    const songs: { name: string; artist_name: string }[] = songsData.data;
    for (let song of songs) {
      this.songs_data.push(new Song(song.name, song.artist_name));
    }
    await MetaCacheService.setSongs(songs, key);
    await this.preloadSongCovers();
  }

  async preloadSongCovers() {
    const promises = this.songs_data.map((song) => this.getSongCover(song));
    await Promise.all(promises);
  }

  async getSongCover(song: Song): Promise<string> {
    const key = `${song.artist}___${song.name}`;
    if (this.songCoverUrls.has(key)) {
      return this.songCoverUrls.get(key)!;
    }
    this.songCoverLoading.set(key, true);
    const url = new URL(`${environment.apiUrl}/api/songs/cover`);
    url.searchParams.append('name', song.name);
    url.searchParams.append('artist_name', song.artist);
    const urlStr = url.toString();
    try {
      const result = await this.imageLoaderService.requestImage(urlStr, { Authorization: `Bearer ${localStorage.getItem('authToken')}` });
      if (result.notFound) {
        const placeholder = 'place_holder.webp';
        this.songCoverUrls.set(key, placeholder);
        this.songCoverLoading.set(key, false);
        return placeholder;
      } else if (result.objectUrl) {
        this.objectUrls.push(result.objectUrl);
        this.songCoverUrls.set(key, result.objectUrl);
        this.songCoverLoading.set(key, false);
        return result.objectUrl;
      } else {
        const placeholder = 'place_holder.webp';
        this.songCoverUrls.set(key, placeholder);
        this.songCoverLoading.set(key, false);
        return placeholder;
      }
    } catch (error) {
      const placeholder = 'place_holder.webp';
      this.songCoverUrls.set(key, placeholder);
      this.songCoverLoading.set(key, false);
      console.error(`[getSongCover] ERROR loading for ${song.artist} - ${song.name}:`, error);
      return placeholder;
    }
  }

  async shiftSpanRight() {
    const new_end: number = this.spanEnd + 36;
    if (new_end >= this.max_count) {
      return;
    }
    this.spanStart = this.spanEnd;
    this.spanEnd = new_end;
    await this.getSongs();
  }

  async shiftSpanLeft() {
    if (this.spanStart == 0) {
      return;
    }
    this.spanEnd = this.spanStart;
    this.spanStart = this.spanStart - 36;
    await this.getSongs();
  }

  async playSong(song: Song) {
    this.musicPlayerService.playSong(song.name, song.artist);
  }

  addToPlaylist(song: Song, event: Event) {
    event.stopPropagation();
    this.playlist_dialog.open(AddToPlaylistPopup, {
      data: { song: song, create_new_playlist: true },
    });
  }

  setSongs(results: SongSearchResult[]) {
    this.songs_data = results.map(r => new Song(r.name, r.artist_name));
    this.songCoverUrls.clear();
    this.songCoverLoading.clear();
    this.preloadSongCovers();
  }

  ngOnDestroy(): void {
    // No worker termination needed
  }
}
