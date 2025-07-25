import {Component, OnInit, OnDestroy, inject, Output, EventEmitter} from '@angular/core';
import { MatCard, MatCardContent } from '@angular/material/card';
import { FaIconComponent } from '@fortawesome/angular-fontawesome';
import { faArrowLeft, faArrowRight } from '@fortawesome/free-solid-svg-icons';
import { Artist } from '../../../data/artist';
import { environment } from '../../../../environments/environment';
import { fetchWithAuth } from '../../../app';
import { Router } from '@angular/router';
import { MetaCacheService } from '../../../services/meta-cache.service';
import { ImageLoaderService } from '../../../services/image-loader.service';

//TODO: this page image loading is broken for some reason need to check if server of client side

@Component({
  selector: 'app-artists',
  standalone: true,
  imports: [
    MatCard,
    MatCardContent,
    FaIconComponent
  ],
  templateUrl: './artists.html',
  styleUrls: ['./artists.scss', '../../shared/card.scss'],
})
export class Artists implements OnInit, OnDestroy {
  private router = inject(Router);
  private max_count: number = 0;
  protected spanStart: number = 0;
  protected spanEnd: number = 36;
  protected artists_data: Artist[] = [];
  protected readonly faArrowRight = faArrowRight;
  protected readonly faArrowLeft = faArrowLeft;
  protected artistCoverUrls = new Map<string, string>();
  protected artistCoverLoading = new Map<string, boolean>();
  private objectUrls: string[] = [];
  @Output() artistSelected = new EventEmitter<string>();

  constructor(private imageLoaderService: ImageLoaderService) {
    // No per-component worker logic needed
  }

  getCoverUrl(artist: Artist): string {
    const key = `${artist.name}`;
    return this.artistCoverUrls.get(key) ?? 'place_holder.webp';
  }

  isCoverLoading(artist: Artist): boolean {
    const key = `${artist.name}`;
    return this.artistCoverLoading.get(key) ?? false;
  }

  async ngOnInit(): Promise<void> {
    try {
      let total = await MetaCacheService.getTotal('artists');
      if (total !== null) {
        this.max_count = total + 36;
      } else {
        const token = localStorage.getItem('authToken');
        const totalResponse = await fetchWithAuth(`${environment.apiUrl}/api/artists/total`, { headers: { Authorization: `Bearer ${token}` } }, this.router);
        const totalData = await totalResponse.json();
        this.max_count = totalData.data.total + 36;
        await MetaCacheService.setTotal('artists', totalData.data.total);
      }
      await this.getArtists();
    } catch (error) {
      console.error(error);
    }
  }

  public async refresh(): Promise<void> {
    await this.ngOnInit();
  }

  async getArtists() {
    this.artists_data = [];
    const key = `artists_${this.spanStart}_${this.spanEnd}`;
    // Try cache first
    const cached = await MetaCacheService.getArtists(key);
    if (cached) {
      for (let artist of cached) {
        const name = artist.artist_name ?? artist.name;
        this.artists_data.push(new Artist(name));
      }
      await this.preloadArtistCovers();
      return;
    }
    // If not cached, fetch from server
    const token = localStorage.getItem('authToken');
    const url = new URL(`${environment.apiUrl}/api/artists`);
    url.searchParams.append('index_start', this.spanStart.toString());
    url.searchParams.append('index_end', this.spanEnd.toString());
    const artistsResponse = await fetchWithAuth(url.toString(), { headers: { Authorization: `Bearer ${token}` } }, this.router);
    const artistsData = await artistsResponse.json();
    const artists: { name: string }[] = artistsData.data;
    for (let artist of artists) {
      this.artists_data.push(new Artist(artist.name));
    }
    await MetaCacheService.setArtists(artists, key);
    await this.preloadArtistCovers();
  }

  async preloadArtistCovers() {
    const promises = this.artists_data.map((artist) => this.getArtistCover(artist));
    await Promise.all(promises);
  }

  async getArtistCover(artist: Artist): Promise<string> {
    const key = `${artist.name}`;
    this.artistCoverLoading.set(key, true);
    const url = new URL(`${environment.apiUrl}/api/artists/cover`);
    url.searchParams.append('name', artist.name);
    const urlStr = url.toString();
    try {
      const result = await this.imageLoaderService.requestImage(urlStr, { Authorization: `Bearer ${localStorage.getItem('authToken')}` });
      if (result.notFound) {
        const placeholder = 'place_holder.webp';
        this.artistCoverUrls.set(key, placeholder);
        this.artistCoverLoading.set(key, false);
        return placeholder;
      } else if (result.objectUrl) {
        this.objectUrls.push(result.objectUrl);
        this.artistCoverUrls.set(key, result.objectUrl);
        this.artistCoverLoading.set(key, false);
        return result.objectUrl;
      } else {
        const placeholder = 'place_holder.webp';
        this.artistCoverUrls.set(key, placeholder);
        this.artistCoverLoading.set(key, false);
        return placeholder;
      }
    } catch (error) {
      const placeholder = 'place_holder.webp';
      this.artistCoverUrls.set(key, placeholder);
      this.artistCoverLoading.set(key, false);
      console.error(`[getArtistCover] ERROR loading for ${artist.name}:`, error);
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
    await this.getArtists();
  }

  async shiftSpanLeft() {
    if (this.spanStart == 0) {
      return;
    }
    this.spanEnd = this.spanStart;
    this.spanStart = this.spanStart - 36;
    await this.getArtists();
  }

  ngOnDestroy(): void {
    // No worker termination needed
  }

  redirectToSongPage(artist: Artist): void {
    this.artistSelected.emit(artist.name);
  }
}
