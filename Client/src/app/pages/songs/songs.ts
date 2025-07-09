import {Component, inject, OnInit} from '@angular/core';
import {MatCard, MatCardActions, MatCardContent} from '@angular/material/card';
import {MatIconButton} from '@angular/material/button';
import {FaIconComponent} from '@fortawesome/angular-fontawesome';
import {faArrowLeft, faArrowRight, faPlus} from '@fortawesome/free-solid-svg-icons';
import {MatSuffix} from '@angular/material/input';
import {MatDialog} from '@angular/material/dialog';
import {AddToPlaylistPopup} from '../../component/add-to-playlist-popup/add-to-playlist-popup';
import {Song} from '../../data/song';
import {ActivatedRoute, Router} from '@angular/router';
import axios from 'axios';
import {routes} from '../../app.routes';
import {environment} from '../../../environments/environment';



@Component({
  selector: 'app-songs',
  imports: [
    MatCard,
    MatCardContent,
    MatCardActions,
    MatIconButton,
    FaIconComponent,
    MatSuffix,
  ],
  templateUrl: './songs.html',
  styleUrl: './songs.css'
})
export class Songs implements OnInit {
  private max_count: number = 0;
  private router = inject(Router);
  readonly songs_data:Song[] = [];
  protected readonly faPlus = faPlus;
  private readonly playlist_dialog: MatDialog = inject(MatDialog);

  async ngOnInit(): Promise<void> {
    try {
      const token = localStorage.getItem('authToken');
      const response = await axios.get(`${environment.apiUrl}/api/songs`, {
        params: {
          index_start: 0,
          index_end: 50
        },
        headers: {
          Authorization: `Bearer ${token}`
        }
      });

      // Handle successful response
      const songs : {song_name: string, artist_name: string}[] = response.data.data;
      for (let song of songs) {
        this.songs_data.push(new Song(song.song_name,song.artist_name));
      }

      // Assign to component property or process further

    } catch (error) {
      console.error(error);
    }
  }

  async playSong(song: Song) {
    console.info(`Playing Song ${song.name} by ${song.artist}`);
    throw new Error("need to sent action to player")
  }

  addToPlaylist(song: Song) {
    this.playlist_dialog.open(AddToPlaylistPopup, {
      data: {song: song},
    });
  }

  getSongCover(song: Song) : string {

    return "place_holder.webp";
  }

  protected readonly faArrowRight = faArrowRight;
  protected readonly faArrowLeft = faArrowLeft;
}
