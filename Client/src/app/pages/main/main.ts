import { Component } from '@angular/core';
import {Artists} from './artists/artists';
import {Songs} from './songs/songs';
import {MatDrawer, MatDrawerContainer, MatDrawerContent} from '@angular/material/sidenav';
import {MatToolbar} from '@angular/material/toolbar';
import {FormControl, FormGroup, ReactiveFormsModule} from '@angular/forms';
import {MatInput} from '@angular/material/input';
import {MatList, MatListItem} from '@angular/material/list';
import {FaIconComponent} from '@fortawesome/angular-fontawesome';
import {faHome, faListCheck, faMicrophone, faMusic, faPlus} from '@fortawesome/free-solid-svg-icons';
import {Playlist} from './playlist/playlist';
import {MusicPlayer} from '../../component/music-player/music-player';

@Component({
  selector: 'app-main',
  imports: [
    Artists,
    Songs,
    MatDrawerContainer,
    MatDrawerContent,
    MatDrawer,
    MatToolbar,
    ReactiveFormsModule,
    MatInput,
    MatList,
    MatListItem,
    FaIconComponent,
    Playlist,
    MusicPlayer,
  ],
  templateUrl: './main.html',
  styleUrl: './main.css'
})
export class Main {
  protected page: string = 'home';
  protected readonly faHome = faHome;
  protected readonly faMicrophone = faMicrophone;
  protected readonly faListCheck = faListCheck;
  protected readonly faMusic = faMusic;
  protected readonly search = new FormGroup({
    query: new FormControl('',[]),
  });

  async sendRequest() {

  }

  setPage(page: string) {
    this.page = page;
  }

  getProfileUrl()  : string {
    return 'place_holder.webp';
  }

}
