import { Component } from '@angular/core';
import {Artists} from '../artists/artists';
import {Songs} from '../songs/songs';

@Component({
  selector: 'app-main',
  imports: [
    Artists,
    Songs
  ],
  templateUrl: './main.html',
  styleUrl: './main.css'
})
export class Main {

}
