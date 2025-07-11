import {Component, OnInit, OnDestroy, inject} from '@angular/core';
import { MatCard, MatCardContent } from '@angular/material/card';
import { FaIconComponent } from '@fortawesome/angular-fontawesome';
import { faArrowLeft, faArrowRight } from '@fortawesome/free-solid-svg-icons';
import { Artist } from '../../data/artist';
import axios from 'axios';
import { environment } from '../../../environments/environment';
import pLimit from 'p-limit';
import {Router} from '@angular/router';

@Component({
  selector: 'app-artists',
  standalone: true,
  imports: [
    MatCard,
    MatCardContent,
    FaIconComponent
  ],
  templateUrl: './artists.html',
  styleUrls: ['./artists.css', '../shared/card.css'],
})
export class Artists implements OnInit, OnDestroy {
  private router = inject(Router);
  private max_count: number = 0;
  protected spanStart: number = 0;
  protected spanEnd: number = 36;
  protected artists_data: Artist[] = [];
  protected readonly faArrowRight = faArrowRight;
  protected readonly faArrowLeft = faArrowLeft;

  private readonly limit = pLimit(5);
  protected artistCoverUrls = new Map<string, string>();
  protected artistCoverLoading = new Map<string, boolean>();
  private objectUrls: string[] = [];

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
      const token = localStorage.getItem('authToken');
      const total = await axios.get(`${environment.apiUrl}/api/artists/total`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });
      this.max_count = total.data.data.total + 36;
      await this.getArtists();
    } catch (error) {
      console.error(error);
    }
  }

  async getArtists() {
    this.artists_data = [];

    const token = localStorage.getItem('authToken');
    const artists_data = await axios.get(`${environment.apiUrl}/api/artists`, {
      params: {
        index_start: this.spanStart,
        index_end: this.spanEnd,
      },
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });

    const artists: { artist_name: string }[] = artists_data.data.data;

    for (let artist of artists) {
      this.artists_data.push(new Artist(artist.artist_name));
    }

    await this.preloadArtistCovers();
  }

  async preloadArtistCovers() {
    const promises = this.artists_data.map((artist) => this.getArtistCover(artist));
    await Promise.all(promises);
  }

  async getArtistCover(artist: Artist): Promise<string> {
    const key = `${artist.name}`;

    if (this.artistCoverUrls.has(key)) {
      return this.artistCoverUrls.get(key)!;
    }

    this.artistCoverLoading.set(key, true);
    const url = await this.limit(async () => {
      try {
        const token = localStorage.getItem('authToken');
        const response = await axios.get(`${environment.apiUrl}/api/artists/cover`, {
          params: {
            artist_name: artist.name,
          },
          headers: {
            Authorization: `Bearer ${token}`,
          },
          responseType: 'arraybuffer',
        });

        if (!response.data || response.data.byteLength === 0) {
          const placeholder = 'place_holder.webp';
          this.artistCoverUrls.set(key, placeholder);
          this.artistCoverLoading.set(key, false);
          return placeholder;
        }

        const contentType = response.headers['content-type'] || 'image/avif';
        const blob = new Blob([response.data], { type: contentType });
        const objectUrl = URL.createObjectURL(blob);
        this.objectUrls.push(objectUrl);
        this.artistCoverUrls.set(key, objectUrl);
        this.artistCoverLoading.set(key, false);
        return objectUrl;
      } catch (error) {
        const placeholder = 'place_holder.webp';
        this.artistCoverUrls.set(key, placeholder);
        this.artistCoverLoading.set(key, false);
        return placeholder;
      }
    });
    this.artistCoverLoading.set(key, false);
    return url;
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
    for (const url of this.objectUrls) {
      URL.revokeObjectURL(url);
    }
    this.objectUrls = [];
  }

  redirectToSongPage(artist: Artist) : void {
    this.router.navigate(['list'], { queryParams: { artist: artist.name } });
  }
}
