import {Component, OnInit} from '@angular/core';
import {ActivatedRoute} from '@angular/router';
import {environment} from '../../../environments/environment';
import {Song} from '../../data/song';
import { fetchWithAuth } from '../../app';
import { Router } from '@angular/router';

@Component({
  selector: 'app-list',
  imports: [],
  templateUrl: './list.html',
  styleUrl: './list.css'
})
export class List implements OnInit {
  private artist_name: string = '';
  readonly songs_list: string[] = [];
  constructor(private route: ActivatedRoute, private router: Router) {}

  async ngOnInit(): Promise<void> {
    this.route.params.subscribe(params => {
      this.artist_name = params['artist_name'] ?? '';
    })
    const token = localStorage.getItem('authToken');
    const url = new URL(`${environment.apiUrl}/api/artists/songs`);
    url.searchParams.append('artist_name', this.artist_name);
    const songsResponse = await fetchWithAuth(url.toString(), { headers: { Authorization: `Bearer ${token}` } }, this.router);
    const songsData = await songsResponse.json();
    const songs: {artist_name: string }[] = songsData.data;

    for (let song of songs) {
      this.songs_list.push(song.artist_name);
    }
  }
}
