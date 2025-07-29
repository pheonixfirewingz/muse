import { Component, OnInit, Input, Output, EventEmitter, OnDestroy } from '@angular/core';
import { FaIconComponent } from '@fortawesome/angular-fontawesome';
import { faPlay } from '@fortawesome/free-solid-svg-icons';
import { Song } from '../../../../data/song';
import { Artist } from '../../../../data/artist';
import { fetchWithAuth } from '../../../../app';
import { environment } from '../../../../../environments/environment';
import {MusicPlayerService} from '../../../../services/music-player.service';
import { ImageLoaderService } from '../../../../services/image-loader.service';

@Component({
  selector: 'app-artist-detail',
  standalone: true,
  imports: [FaIconComponent],
  templateUrl: './artist-detail.html',
  styleUrls: ['./artist-detail.scss']
})
export class ArtistDetail implements OnInit, OnDestroy {
  @Input() artistName!: string;
  @Output() back = new EventEmitter<void>();
  artist: Artist | null = null;
  songs: Song[] = [];
  artistCoverUrl: string = 'place_holder.webp';
  faPlay = faPlay;
  loadingSongs = true;
  loadingCover = true;
  private objectUrls: string[] = [];

  constructor(private musicPlayerService: MusicPlayerService, private imageLoaderService: ImageLoaderService) {
    // No per-component worker logic needed
  }

  async ngOnInit() {
    if (!this.artistName) return;
    this.artist = new Artist(this.artistName);
    this.fetchSongs(this.artistName);
    this.fetchArtistCover(this.artistName);
  }

  ngOnDestroy() {
    // No worker termination needed
  }

  fetchSongs(artistName: string) {
    this.loadingSongs = true;
    const token = localStorage.getItem('authToken');
    const url = new URL(`${environment.apiUrl}/api/artists/songs`);
    url.searchParams.append('name', artistName);
    fetchWithAuth(url.toString(), { headers: { Authorization: `Bearer ${token}` } })
      .then(response => response.json())
      .then(data => {
        this.songs = (data.data || []).map((s: any) => new Song(s.name, artistName));
        this.loadingSongs = false;
      })
      .catch(() => {
        this.songs = [];
        this.loadingSongs = false;
      });
  }

  fetchArtistCover(artistName: string) {
    this.loadingCover = true;
    const url = new URL(`${environment.apiUrl}/api/artists/cover`);
    url.searchParams.append('name', artistName);
    const urlStr = url.toString();
    (async () => {
      try {
        const result = await this.imageLoaderService.requestImage(urlStr, { Authorization: `Bearer ${localStorage.getItem('authToken')}` });
        if (result.notFound) {
          this.artistCoverUrl = 'place_holder.webp';
        } else if (result.objectUrl) {
          this.objectUrls.push(result.objectUrl);
          this.artistCoverUrl = result.objectUrl;
        } else {
          this.artistCoverUrl = 'place_holder.webp';
        }
        this.loadingCover = false;
      } catch (error) {
        console.error(`[ArtistDetailCover] ERROR loading for ${artistName}:`, error);
        this.artistCoverUrl = 'place_holder.webp';
        this.loadingCover = false;
      }
    })();
  }

  playAllSongs() {
    this.musicPlayerService.playQueue(this.songs);
  }

  playSong(song: Song) {
    this.musicPlayerService.playSong(song.name,song.artist);
  }

  goBack() {
    this.back.emit();
  }
}
