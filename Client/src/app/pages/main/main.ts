import { Component } from '@angular/core';
import {Songs} from '../songs/songs';

@Component({
  selector: 'app-main',
  imports: [
    Songs
  ],
  templateUrl: './main.html',
  styleUrl: './main.css'
})
export class Main {

}
