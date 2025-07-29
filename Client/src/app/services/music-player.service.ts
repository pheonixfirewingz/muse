import { Injectable } from '@angular/core';
import { Subject } from 'rxjs';
import { environment } from '../../environments/environment';
import { fetchWithAuth } from '../app';

export type MusicPlayerCommand =
  | { type: 'play' }
  | { type: 'pause' }
  | { type: 'load', src: string }
  | { type: 'playSong', name: string, artist: string }
  | { type: 'queue', songs: { name: string, artist: string }[] };

export interface SongSearchResult {
  name: string;
  artist_name: string;
}

@Injectable({ providedIn: 'root' })
export class MusicPlayerService {
  private commandSubject = new Subject<MusicPlayerCommand>();
  command$ = this.commandSubject.asObservable();
  playSong(name: string, artist: string) {
    this.commandSubject.next({ type: 'playSong', name, artist });
  }
  playQueue(songs: { name: string, artist: string }[]) {
    this.commandSubject.next({ type: 'queue', songs });
  }

  async fuzzySearchSongs(query: string): Promise<SongSearchResult[]> {
    const token = localStorage.getItem('authToken');
    const url = new URL(`${environment.apiUrl}/api/songs/search`);
    url.searchParams.append('query', query);
    const response = await fetchWithAuth(url.toString(), { headers: { Authorization: `Bearer ${token}` } });
    if (!response.ok) throw new Error('Search failed');
    const data = await response.json();
    return data.data || [];
  }
}
