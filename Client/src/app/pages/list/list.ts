import {Component, OnInit} from '@angular/core';
import {ActivatedRoute} from '@angular/router';
import {environment} from '../../../environments/environment';
import { fetchWithAuth } from '../../app';
import { Router } from '@angular/router';
import { MatList, MatListItem} from '@angular/material/list';

@Component({
  selector: 'app-list',
  imports: [
    MatListItem,
    MatList
  ],
  templateUrl: './list.html',
  styleUrl: './list.css'
})
export class List implements OnInit {
  private artist_name: string = '';
  songs_list: string[] = [];
  constructor(private route: ActivatedRoute, private router: Router) {}

  async ngOnInit(): Promise<void> {
    this.route.queryParams.subscribe(async params => {
      this.artist_name = params['artist_name'];
      if (!this.artist_name) return;
      this.songs_list.length = 0;
      const token = localStorage.getItem('authToken');
      const url = new URL(`${environment.apiUrl}/api/artists/songs`);
      url.searchParams.append('name', this.artist_name);
      const songsResponse = await fetchWithAuth(url.toString(), { headers: { Authorization: `Bearer ${token}` } }, this.router);
      const songsData = await songsResponse.json();
      this.songs_list = songsData.data;
    });
  }
}
