import {Component, OnInit} from '@angular/core';
import {ActivatedRoute} from '@angular/router';
import axios from 'axios';
import {environment} from '../../../environments/environment';
import {Song} from '../../data/song';

@Component({
  selector: 'app-list',
  imports: [],
  templateUrl: './list.html',
  styleUrl: './list.css'
})
export class List implements OnInit {
  private artist_name: string = '';
  readonly songs_list: string[] = [];
  constructor(private route: ActivatedRoute) {}

  async ngOnInit(): Promise<void> {
    this.route.params.subscribe(params => {
      this.artist_name = params['artist_name'] ?? '';
    })
    const token = localStorage.getItem('authToken');
    const songs_data = await axios.get(`${environment.apiUrl}/api/artists/songs`, {
      params: {
        artist_name: this.artist_name,
      },
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
    const songs: {artist_name: string }[] = songs_data.data.data;

    for (let song of songs) {
      this.songs_list.push(song.artist_name);
    }
  }
}
