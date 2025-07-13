import { Component } from '@angular/core';
import {MatSlider, MatSliderThumb} from '@angular/material/slider';
import {FaIconComponent} from '@fortawesome/angular-fontawesome';
import {MatButton} from '@angular/material/button';
import {faBackward, faForward, faPause, faPlay, faVolumeHigh, faVolumeOff} from '@fortawesome/free-solid-svg-icons';
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
  private currentBlobUrl: string | null = null;
  private queue: { name: string, artist: string }[] = [];
  private queueIndex: number = 0;

  constructor() {
    this.audio = new Audio();
    this.audio.addEventListener('ended', () => {
      this.is_playing = false;
      void this.playNextInQueue();
    });
    this.audio.addEventListener('timeupdate', () => {
      this.bar_progress = this.audio.currentTime;
      this.duration = this.audio.duration || 0;
      this.time = this.formatTime(this.audio.currentTime) + '/' + this.formatTime(this.audio.duration);
    });
    // Subscribe to global player commands
    const playerService = inject(MusicPlayerService);
    playerService.command$.subscribe((cmd: MusicPlayerCommand) => {
      switch (cmd.type) {
        case 'playSong': {
          // Add the song after the current song in the queue, unless it's already there
          const exists = this.queue.some(q => q.name === cmd.name && q.artist === cmd.artist);
          if (!exists) {
            this.queue.splice(this.queueIndex + 1, 0, { name: cmd.name, artist: cmd.artist });
          }
          this.queueIndex++;
          const { name, artist } = this.queue[this.queueIndex];
          void this.fetchAndPlay(name, artist);
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
    const value = typeof event === 'number' ? event : event?.value;
    if (typeof value === 'number') {
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

  async fetchAndPlay(name: string, artist: string) {
    const token = localStorage.getItem('authToken');
    const url = new URL(`${environment.apiUrl}/api/stream`);
    url.searchParams.append('name', name);
    url.searchParams.append('artist', artist);
    try {
      const response = await fetchWithAuth(url.toString(), { headers: { Authorization: `Bearer ${token}` } });
      if (!response.ok) throw new Error('Failed to fetch audio stream');
      const blob = await response.blob();
      if (this.currentBlobUrl) {
        URL.revokeObjectURL(this.currentBlobUrl);
      }
      this.currentBlobUrl = URL.createObjectURL(blob);
      this.load(this.currentBlobUrl);
      this.play();
    } catch (e) {
      console.error('Error fetching audio:', e);
    }
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
}
