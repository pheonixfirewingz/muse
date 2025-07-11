import { Component } from '@angular/core';
import {Artists} from './artists/artists';
import {Songs} from './songs/songs';
import {MatDrawer, MatDrawerContainer, MatDrawerContent} from '@angular/material/sidenav';

@Component({
  selector: 'app-main',
  imports: [
    Artists,
    Songs,
    MatDrawerContainer,
    MatDrawerContent,
    MatDrawer
  ],
  templateUrl: './main.html',
  styleUrl: './main.css'
})
export class Main {

}
