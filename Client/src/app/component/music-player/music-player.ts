import { Component } from '@angular/core';
import {MatSlider, MatSliderThumb} from '@angular/material/slider';

@Component({
  selector: 'app-music-player',
  imports: [
    MatSlider,
    MatSliderThumb
  ],
  templateUrl: './music-player.html',
  styleUrl: './music-player.sass'
})
export class MusicPlayer {
  protected bar_progress: number = 0;

}
