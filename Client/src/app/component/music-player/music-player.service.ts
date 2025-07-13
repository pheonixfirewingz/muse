import { Injectable } from '@angular/core';
import { Subject } from 'rxjs';

export type MusicPlayerCommand =
  | { type: 'play' }
  | { type: 'pause' }
  | { type: 'load', src: string }
  | { type: 'playSong', name: string, artist: string }
  | { type: 'queue', songs: { name: string, artist: string }[] };

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
} 