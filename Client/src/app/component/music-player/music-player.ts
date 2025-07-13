import { Component } from '@angular/core';
import {MatSlider, MatSliderThumb} from '@angular/material/slider';
import {FaIconComponent} from '@fortawesome/angular-fontawesome';
import {MatButton} from '@angular/material/button';
import {faBackward, faForward, faPause, faPlay, faVolumeHigh, faVolumeOff, faRedo, faRedoAlt} from '@fortawesome/free-solid-svg-icons';
import { FormsModule } from '@angular/forms';
import { MusicPlayerService, MusicPlayerCommand } from './music-player.service';
import { inject } from '@angular/core';
import { environment } from '../../../environments/environment';
import { fetchWithAuth } from '../../app';

@Component({
  selector: 'app-music-player',
  imports: [
    MatSlider,
    MatSliderThumb,
    FaIconComponent,
    MatButton,
    FormsModule
  ],
  templateUrl: './music-player.html',
  styleUrl: './music-player.scss'
})
export class MusicPlayer {
  protected bar_progress: number = 0;
  protected duration: number = 0;
  private is_playing: boolean = false;
  protected time: string = "0:00";
  private audio: HTMLAudioElement;
  private currentSrc: string = "";
  protected readonly faForward = faForward;
  protected readonly faBackward = faBackward;
  protected readonly faRedo = faRedo;
  protected readonly faRedoAlt = faRedoAlt;
  private currentBlobUrl: string | null = null;
  private queue: { name: string, artist: string }[] = [];
  private queueIndex: number = 0;
  private isLoopEnabled: boolean = false;
  private currentSongCache: { name: string, artist: string, blob: Blob } | null = null;

  // Example current song object
  currentSong: { name: string, artist: string, coverUrl: string } | null = null;

  // Expose these for the template
  get coverUrl(): string {
    return this.currentSong?.coverUrl || 'assets/default-cover.png'; // fallback image
  }

  get songName(): string {
    return this.currentSong?.name || 'No Song';
  }

  get artistName(): string {
    return this.currentSong?.artist || 'Unknown Artist';
  }

  private imageLoaderWorker: Worker | null = null;
  private workerCallbacks = new Map<string, (result: any) => void>();

  constructor() {
    this.audio = new Audio();
    this.audio.addEventListener('ended', () => {
      this.is_playing = false;
      if (this.isLoopEnabled && this.currentSongCache) {
        // Loop the current song using cached data
        this.replayCurrentSong();
      } else {
        void this.playNextInQueue();
      }
    });
    this.audio.addEventListener('timeupdate', () => {
      this.bar_progress = this.audio.currentTime;
      this.duration = this.audio.duration || 0;
      this.time = this.formatTime(this.audio.currentTime) + '/' + this.formatTime(this.audio.duration);
    });
    if (typeof window !== 'undefined' && typeof Worker !== 'undefined') {
      this.imageLoaderWorker = new Worker(new URL('../../pages/shared/image-loader.worker.ts', import.meta.url), { type: 'module' });
      this.imageLoaderWorker.onmessage = (event: MessageEvent) => {
        const { url } = event.data;
        const cb = this.workerCallbacks.get(url);
        if (cb) {
          cb(event.data);
          this.workerCallbacks.delete(url);
        }
      };
    }
    // Subscribe to global player commands
    const playerService = inject(MusicPlayerService);
    playerService.command$.subscribe((cmd: MusicPlayerCommand) => {
      switch (cmd.type) {
        case 'playSong': {
          // If queue is empty, add the song and play it
          if (this.queue.length === 0) {
            this.queue = [{ name: cmd.name, artist: cmd.artist }];
            this.queueIndex = 0;
          } else {
            // Add the song after the current song in the queue, unless it's already there
            const exists = this.queue.some(q => q.name === cmd.name && q.artist === cmd.artist);
            if (!exists) {
              this.queue.splice(this.queueIndex + 1, 0, { name: cmd.name, artist: cmd.artist });
            }
            this.queueIndex++;
          }
          
          // Now safely access the queue element
          if (this.queueIndex < this.queue.length) {
            const { name, artist } = this.queue[this.queueIndex];
            void this.fetchAndPlay(name, artist);
          }
          break;
        }
        case 'queue':
          this.queue = cmd.songs.slice();
          this.queueIndex = 0;
          if (this.queue.length > 0) {
            const { name, artist } = this.queue[0];
            void this.fetchAndPlay(name, artist);
          }
          break;
      }
    });
  }

  onSliderChange(event: any) {
    console.log('Slider change event:', event);
    let value: number;
    
    // Handle input event from slider thumb
    if (event && event.target && typeof event.target.value === 'string') {
      value = parseFloat(event.target.value);
    } else if (typeof event === 'number') {
      value = event;
    } else if (event && typeof event.value === 'number') {
      value = event.value;
    } else {
      console.warn('Unexpected slider event format:', event);
      return;
    }
    
    console.log('Setting audio time to:', value);
    if (this.audio && !isNaN(value) && value >= 0) {
      this.audio.currentTime = value;
      this.bar_progress = value;
    }
  }

  private formatTime(seconds: number): string {
    if (!seconds || isNaN(seconds)) return '0:00';
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  public play() {
    if (this.currentSrc) {
      void this.audio.play();
      this.is_playing = true;
    }
  }

  public pause() {
    this.audio.pause();
    this.is_playing = false;
  }

  public load(src: string) {
    this.currentSrc = src;
    this.audio.src = src;
    this.audio.load();
    this.is_playing = false;
    this.bar_progress = 0;
    this.duration = 0;
    this.time = '0:00';
  }

  /*public setVolume(volume: number) {
    this.audio.volume = volume;
  }*/

  isMute() {
    return this.audio.muted ? faVolumeOff: faVolumeHigh;
  }

  toggleMute() {
    this.audio.muted = !this.audio.muted;
  }

  isPlaying() {
    return this.is_playing ? faPause : faPlay;
  }

  togglePlaying() {
    if (this.is_playing) {
      this.pause();
    } else {
      this.play();
    }
  }

  private async getSongCoverUrl(name: string, artist: string): Promise<string> {
    const key = `${artist}___${name}`;
    const url = new URL(`${environment.apiUrl}/api/songs/cover`);
    url.searchParams.append('name', name);
    url.searchParams.append('artist_name', artist);
    const urlStr = url.toString();
    if (this.imageLoaderWorker) {
      return new Promise((resolve) => {
        this.workerCallbacks.set(urlStr, (result) => {
          if (result.notFound) {
            resolve('place_holder.webp');
          } else if (result.objectUrl) {
            resolve(result.objectUrl);
          } else {
            resolve('place_holder.webp');
          }
        });
        this.imageLoaderWorker!.postMessage({ url: urlStr, headers: { Authorization: `Bearer ${localStorage.getItem('authToken')}` } });
      });
    } else {
      // fallback to direct fetch if worker not available
      try {
        const token = localStorage.getItem('authToken');
        const response = await fetch(urlStr, { headers: { Authorization: `Bearer ${token}` } });
        if (!response.ok) return 'place_holder.webp';
        const arrayBuffer = await response.arrayBuffer();
        if (!arrayBuffer || arrayBuffer.byteLength === 0) return 'place_holder.webp';
        const contentType = response.headers.get('content-type') || 'image/avif';
        const blob = new Blob([arrayBuffer], { type: contentType });
        return URL.createObjectURL(blob);
      } catch {
        return 'place_holder.webp';
      }
    }
  }

  async fetchAndPlay(name: string, artist: string) {
    // Check if we have this song cached and loop is enabled
    if (this.isLoopEnabled && this.currentSongCache && 
        this.currentSongCache.name === name && this.currentSongCache.artist === artist) {
      this.replayCurrentSong();
      return;
    }

    const token = localStorage.getItem('authToken');
    const url = new URL(`${environment.apiUrl}/api/stream`);
    url.searchParams.append('name', name);
    url.searchParams.append('artist', artist);
    try {
      const response = await fetchWithAuth(url.toString(), { headers: { Authorization: `Bearer ${token}` } });
      if (!response.ok) throw new Error('Failed to fetch audio stream');
      const blob = await response.blob();
      
      // Cache the song data for looping
      this.currentSongCache = { name, artist, blob };
      
      if (this.currentBlobUrl) {
        URL.revokeObjectURL(this.currentBlobUrl);
      }
      this.currentBlobUrl = URL.createObjectURL(blob);
      this.load(this.currentBlobUrl);
      // Fetch cover art using image worker
      const coverUrl = await this.getSongCoverUrl(name, artist);
      this.currentSong = { name, artist, coverUrl };
      this.play();
    } catch (e) {
      console.error('Error fetching audio:', e);
    }
  }

  private replayCurrentSong() {
    if (this.currentSongCache && this.currentBlobUrl) {
      // Reset audio to beginning and play
      this.audio.currentTime = 0;
      this.bar_progress = 0;
      this.play();
    }
  }

  toggleLoop() {
    this.isLoopEnabled = !this.isLoopEnabled;
    console.log('Loop enabled:', this.isLoopEnabled);
  }

  isLoopActive() {
    return this.isLoopEnabled;
  }

  getLoopIcon() {
    return this.isLoopEnabled ? this.faRedoAlt : this.faRedo;
  }

  private async playNextInQueue() {
    if (this.queue.length > 0 && this.queueIndex + 1 < this.queue.length) {
      this.queueIndex++;
      // Remove history beyond 3 previous songs
      if (this.queueIndex > 3) {
        this.queue.splice(0, this.queueIndex - 3);
        this.queueIndex = 3;
      }
      const { name, artist } = this.queue[this.queueIndex];
      void this.fetchAndPlay(name, artist);
    } else {
      this.queue = [];
      this.queueIndex = 0;
    }
  }

  next() {
    void this.playNextInQueue();
  }

  previous() {
    const minIndex = Math.max(0, this.queueIndex - 3);
    if (this.queue.length > 0 && this.queueIndex > minIndex) {
      this.queueIndex--;
      const { name, artist } = this.queue[this.queueIndex];
      void this.fetchAndPlay(name, artist);
    }
  }

  // Check if previous button should be enabled
  canGoPrevious(): boolean {
    const minIndex = Math.max(0, this.queueIndex - 3);
    return this.queue.length > 0 && this.queueIndex > minIndex;
  }

  // Check if next button should be enabled
  canGoNext(): boolean {
    return this.queue.length > 0 && this.queueIndex + 1 < this.queue.length;
  }
}
