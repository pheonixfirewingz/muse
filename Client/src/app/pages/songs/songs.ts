import { Component, inject, OnInit, OnDestroy } from '@angular/core';
import { MatCard, MatCardActions, MatCardContent } from '@angular/material/card';
import { MatIconButton } from '@angular/material/button';
import { FaIconComponent } from '@fortawesome/angular-fontawesome';
import { faArrowLeft, faArrowRight, faPlus } from '@fortawesome/free-solid-svg-icons';
import { MatSuffix } from '@angular/material/input';
import { MatDialog } from '@angular/material/dialog';
import { AddToPlaylistPopup } from '../../component/add-to-playlist-popup/add-to-playlist-popup';
import { Song } from '../../data/song';
import axios from 'axios';
import { environment } from '../../../environments/environment';
import pLimit from 'p-limit';

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
  styleUrls: ['./songs.css', '../shared/card.css'],
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

  private readonly limit = pLimit(5);
  protected songCoverUrls = new Map<string, string>();
  protected songCoverLoading = new Map<string, boolean>();
  private objectUrls: string[] = [];

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
      const token = localStorage.getItem('authToken');
      const total = await axios.get(`${environment.apiUrl}/api/songs/total`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });
      this.max_count = total.data.data.total + 36;
      await this.getSongs();
    } catch (error) {
      console.error(error);
    }
  }

  async getSongs() {
    this.songs_data = [];

    const token = localStorage.getItem('authToken');
    const songs_data = await axios.get(`${environment.apiUrl}/api/songs`, {
      params: {
        index_start: this.spanStart,
        index_end: this.spanEnd,
      },
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });

    const songs: { song_name: string; artist_name: string }[] = songs_data.data.data;

    for (let song of songs) {
      this.songs_data.push(new Song(song.song_name, song.artist_name));
    }

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
    const url = await this.limit(async () => {
      try {
        const token = localStorage.getItem('authToken');
        const response = await axios.get(`${environment.apiUrl}/api/songs/cover`, {
          params: {
            artist_name: song.artist,
            song_name: song.name,
          },
          headers: {
            Authorization: `Bearer ${token}`,
          },
          responseType: 'arraybuffer',
        });

        if (!response.data || response.data.byteLength === 0) {
          const placeholder = 'place_holder.webp';
          this.songCoverUrls.set(key, placeholder);
          this.songCoverLoading.set(key, false);
          return placeholder;
        }

        const contentType = response.headers['content-type'] || 'image/avif';
        const blob = new Blob([response.data], { type: contentType });
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
    });
    this.songCoverLoading.set(key, false);
    return url;
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
  }
}
