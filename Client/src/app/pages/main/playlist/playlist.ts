import {Component, inject, OnInit} from '@angular/core';
import { PlaylistData } from '../../../data/playlist';
import { fetchWithAuth } from '../../../app';
import { environment } from '../../../../environments/environment';
import {FaIconComponent} from '@fortawesome/angular-fontawesome';
import {MatCard, MatCardContent} from '@angular/material/card';
import {faListCheck, faPlus} from '@fortawesome/free-solid-svg-icons';
import {faArrowRight, faArrowLeft} from '@fortawesome/free-solid-svg-icons';
import {AddToPlaylistPopup} from '../../../component/add-to-playlist-popup/add-to-playlist-popup';
import {MatDialog} from '@angular/material/dialog';

@Component({
  selector: 'app-playlist',
  templateUrl: './playlist.html',
  styleUrls: ['./playlist.scss', '../../shared/card.scss'],
  imports: [
    MatCard,
    MatCardContent,
    FaIconComponent,
  ]
})
export class Playlist implements OnInit {
  public privatePlaylists: PlaylistData[] = [];
  public publicPlaylists: PlaylistData[] = [];
  public privateMaxCount: number = 0;
  public publicMaxCount: number = 0;
  public privateSpanStart: number = 0;
  public privateSpanEnd: number = 36;
  public publicSpanStart: number = 0;
  public publicSpanEnd: number = 36;

  private readonly playlist_dialog: MatDialog = inject(MatDialog);
  protected readonly faArrowRight = faArrowRight;
  protected readonly faArrowLeft = faArrowLeft;

  async ngOnInit(): Promise<void> {
    await this.fetchCounts();
    await this.fetchPlaylists();
  }

  async fetchCounts() {
    const token = localStorage.getItem('authToken');
    if (!token) {
      window.location.href = '/login';
      return;
    }
    // Private total
    try {
      const privateTotalRes = await fetchWithAuth(
        `${environment.apiUrl}/api/playlists/private/total`,
        { headers: { Authorization: `Bearer ${token}` } }
      );
      const privateTotalData = await privateTotalRes.json();
      this.privateMaxCount = (privateTotalData.data?.total ?? 0) + 36;
    } catch (e) {
      this.privateMaxCount = 36;
    }
    // Public total
    try {
      const publicTotalRes = await fetchWithAuth(
        `${environment.apiUrl}/api/playlists/public/total`,
        { headers: { Authorization: `Bearer ${token}` } }
      );
      const publicTotalData = await publicTotalRes.json();
      this.publicMaxCount = (publicTotalData.data?.total ?? 0) + 36;
    } catch (e) {
      this.publicMaxCount = 36;
    }
  }

  async fetchPlaylists() {
    const token = localStorage.getItem('authToken');
    if (!token) {
      window.location.href = '/login';
      return;
    }
    // Fetch private playlists (paginated)
    try {
      const privateUrl = new URL(`${environment.apiUrl}/api/playlists/private`);
      privateUrl.searchParams.append('index_start', this.privateSpanStart.toString());
      privateUrl.searchParams.append('index_end', this.privateSpanEnd.toString());
      const privateRes = await fetchWithAuth(
        privateUrl.toString(),
        { headers: { Authorization: `Bearer ${token}` } }
      );
      const privateData = await privateRes.json();
      this.privatePlaylists = (privateData.data || []).map((pl: any) => new PlaylistData(pl.name, pl.owner, false));
    } catch (e) {
      this.privatePlaylists = [];
    }
    // Fetch public playlists (paginated)
    try {
      const publicUrl = new URL(`${environment.apiUrl}/api/playlists/public`);
      publicUrl.searchParams.append('index_start', this.publicSpanStart.toString());
      publicUrl.searchParams.append('index_end', this.publicSpanEnd.toString());
      const publicRes = await fetchWithAuth(
        publicUrl.toString(),
        { headers: { Authorization: `Bearer ${token}` } }
      );
      const publicData = await publicRes.json();
      this.publicPlaylists = (publicData.data || []).map((pl: any) => new PlaylistData(pl.name, pl.owner, true));
    } catch (e) {
      this.publicPlaylists = [];
    }
  }

  async shiftPrivateRight() {
    const new_end: number = this.privateSpanEnd + 36;
    if (new_end >= this.privateMaxCount) {
      return;
    }
    this.privateSpanStart = this.privateSpanEnd;
    this.privateSpanEnd = new_end;
    await this.fetchPlaylists();
  }

  async shiftPrivateLeft() {
    if (this.privateSpanStart == 0) {
      return;
    }
    this.privateSpanEnd = this.privateSpanStart;
    this.privateSpanStart = this.privateSpanStart - 36;
    await this.fetchPlaylists();
  }

  async shiftPublicRight() {
    const new_end: number = this.publicSpanEnd + 36;
    if (new_end >= this.publicMaxCount) {
      return;
    }
    this.publicSpanStart = this.publicSpanEnd;
    this.publicSpanEnd = new_end;
    await this.fetchPlaylists();
  }

  async shiftPublicLeft() {
    if (this.publicSpanStart == 0) {
      return;
    }
    this.publicSpanEnd = this.publicSpanStart;
    this.publicSpanStart = this.publicSpanStart - 36;
    await this.fetchPlaylists();
  }

  protected readonly faListCheck = faListCheck;

  onAddPlaylist() {
    this.playlist_dialog.open(AddToPlaylistPopup, {
      data: { song: null, create_new_playlist: false },
    });
  }
}
