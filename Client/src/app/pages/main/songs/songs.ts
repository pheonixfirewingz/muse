import { Component, inject, OnInit, OnDestroy } from '@angular/core';
import { MatCard, MatCardActions, MatCardContent } from '@angular/material/card';
import { MatIconButton } from '@angular/material/button';
import { FaIconComponent } from '@fortawesome/angular-fontawesome';
import { faArrowLeft, faArrowRight, faPlus } from '@fortawesome/free-solid-svg-icons';
import { MatSuffix } from '@angular/material/input';
import { MatDialog } from '@angular/material/dialog';
import { AddToPlaylistPopup } from '../../../component/add-to-playlist-popup/add-to-playlist-popup';
import { Song } from '../../../data/song';
import { environment } from '../../../../environments/environment';
import { fetchWithAuth } from '../../../app';
import { Router } from '@angular/router';
import { MetaCacheService } from '../../shared/meta-cache.service';

@Component({
  selector: 'app-songs',
  standalone: true,
  imports: [
    MatCard,
    MatCardContent,
    MatCardActions,
    MatIconButton,
    FaIconComponent,
    MatSuffix,
  ],
  templateUrl: './songs.html',
  styleUrls: ['./songs.css', '../../shared/card.css'],
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

  private imageLoaderWorker: Worker | null = null;
  private workerCallbacks = new Map<string, (result: any) => void>();

  constructor() {
    if (typeof window !== 'undefined' && typeof Worker !== 'undefined') {
      this.imageLoaderWorker = new Worker(new URL('../../shared/image-loader.worker.ts', import.meta.url), { type: 'module' });
      this.imageLoaderWorker.onmessage = (event: MessageEvent) => {
        const { url } = event.data;
        const cb = this.workerCallbacks.get(url);
        if (cb) {
          cb(event.data);
          this.workerCallbacks.delete(url);
        }
      };
    }
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
    if (this.imageLoaderWorker) {
      return new Promise((resolve) => {
        this.workerCallbacks.set(urlStr, (result) => {
          if (result.notFound) {
            const placeholder = 'place_holder.webp';
            this.songCoverUrls.set(key, placeholder);
            this.songCoverLoading.set(key, false);
            resolve(placeholder);
          } else if (result.objectUrl) {
            this.objectUrls.push(result.objectUrl);
            this.songCoverUrls.set(key, result.objectUrl);
            this.songCoverLoading.set(key, false);
            resolve(result.objectUrl);
          } else {
            const placeholder = 'place_holder.webp';
            this.songCoverUrls.set(key, placeholder);
            this.songCoverLoading.set(key, false);
            resolve(placeholder);
          }
        });
        this.imageLoaderWorker!.postMessage({ url: urlStr, headers: { Authorization: `Bearer ${localStorage.getItem('authToken')}` } });
      });
    } else {
      // fallback to direct fetch if worker not available
      try {
        const token = localStorage.getItem('authToken');
        const response = await fetch(url.toString(), {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        });
        if (!response.ok) {
          const placeholder = 'place_holder.webp';
          this.songCoverUrls.set(key, placeholder);
          this.songCoverLoading.set(key, false);
          return placeholder;
        }
        const arrayBuffer = await response.arrayBuffer();
        if (!arrayBuffer || arrayBuffer.byteLength === 0) {
          const placeholder = 'place_holder.webp';
          this.songCoverUrls.set(key, placeholder);
          this.songCoverLoading.set(key, false);
          return placeholder;
        }
        const contentType = response.headers.get('content-type') || 'image/avif';
        const blob = new Blob([arrayBuffer], { type: contentType });
        const objectUrl = URL.createObjectURL(blob);
        this.objectUrls.push(objectUrl);
        this.songCoverUrls.set(key, objectUrl);
        this.songCoverLoading.set(key, false);
        return objectUrl;
      } catch (error) {
        const placeholder = 'place_holder.webp';
        this.songCoverUrls.set(key, placeholder);
        this.songCoverLoading.set(key, false);
        return placeholder;
      }
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
    console.info(`Playing Song ${song.name} by ${song.artist}`);
    throw new Error('need to send action to player');
  }

  addToPlaylist(song: Song) {
    this.playlist_dialog.open(AddToPlaylistPopup, {
      data: { song: song },
    });
  }

  ngOnDestroy(): void {
    for (const url of this.objectUrls) {
      URL.revokeObjectURL(url);
    }
    this.objectUrls = [];
    if (this.imageLoaderWorker) {
      this.imageLoaderWorker.terminate();
      this.imageLoaderWorker = null;
    }
  }
}
