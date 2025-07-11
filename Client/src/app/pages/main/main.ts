import { Component } from '@angular/core';
import {Artists} from './artists/artists';
import {Songs} from './songs/songs';
import {MatDrawer, MatDrawerContainer, MatDrawerContent} from '@angular/material/sidenav';
import {MatToolbar} from '@angular/material/toolbar';
import {MatButton} from '@angular/material/button';

@Component({
  selector: 'app-main',
  imports: [
    Artists,
    Songs,
    MatDrawerContainer,
    MatDrawerContent,
    MatDrawer,
    MatToolbar,
    MatButton
  ],
  templateUrl: './main.html',
  styleUrl: './main.css'
})
export class Main {

}
