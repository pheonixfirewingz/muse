import {Component, inject, OnInit} from '@angular/core';
import {Song} from '../../data/song';
import {
  MAT_DIALOG_DATA,
  MatDialogActions,
  MatDialogClose,
  MatDialogContent,
  MatDialogTitle
} from '@angular/material/dialog';
import {faPlus, faX} from '@fortawesome/free-solid-svg-icons';
import {FaIconComponent} from '@fortawesome/angular-fontawesome';
import {MatIconButton} from '@angular/material/button';
import {MatFormField, MatLabel, MatSuffix} from '@angular/material/input';
import {MatOption} from '@angular/material/core';
import {MatSelect, MatSelectModule} from '@angular/material/select';
import {MatCard, MatCardContent} from '@angular/material/card';

export class PlaylistPopupData {
  song: Song | null = null;
  create_new_playlist: boolean | null = null;
  protected readonly faPlus = faPlus;
}

@Component({
  selector: 'app-add-to-playlist-popup',
  imports: [
    MatDialogTitle,
    MatDialogActions,
    FaIconComponent,
    MatIconButton,
    MatSuffix,
    MatDialogContent,
    MatDialogClose,
    MatOption,
    MatSelect,
    MatLabel,
    MatFormField,
  ],
  templateUrl: './add-to-playlist-popup.html',
  styleUrl: './add-to-playlist-popup.scss'
})
export class AddToPlaylistPopup {
  readonly data: PlaylistPopupData = inject<PlaylistPopupData>(MAT_DIALOG_DATA);
  readonly playlist: string[] = [];
  protected readonly faX = faX;
  protected readonly faPlus = faPlus;

  setCreatePlaylist() {
    this.data.create_new_playlist = false;
  }
}
