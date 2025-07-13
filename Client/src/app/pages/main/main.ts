import { Component, OnInit, OnDestroy, HostListener } from '@angular/core';
import {Artists} from './artists/artists';
import {Songs} from './songs/songs';
import {MatDrawer, MatDrawerContainer, MatDrawerContent} from '@angular/material/sidenav';
import {MatToolbar} from '@angular/material/toolbar';
import {FormControl, FormGroup, ReactiveFormsModule} from '@angular/forms';
import {MatFormField, MatInput, MatLabel} from '@angular/material/input';
import {MatList, MatListItem} from '@angular/material/list';
import {FaIconComponent} from '@fortawesome/angular-fontawesome';
import {faHome, faListCheck, faMicrophone, faMusic, faPlus, faBars} from '@fortawesome/free-solid-svg-icons';
import {Playlist} from './playlist/playlist';
import {MusicPlayer} from '../../component/music-player/music-player';
import {MatIconButton} from '@angular/material/button';

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
    MatIconButton,
  ],
  templateUrl: './main.html',
  styleUrl: './main.scss'
})
export class Main implements OnInit, OnDestroy {
  protected page: string = 'home';
  protected readonly faHome = faHome;
  protected readonly faMicrophone = faMicrophone;
  protected readonly faListCheck = faListCheck;
  protected readonly faMusic = faMusic;
  protected readonly faBars = faBars;
  protected readonly search = new FormGroup({
    query: new FormControl('',[]),
  });

  // Drawer state
  protected isDrawerOpen = true;
  protected isMobile = false;
  private readonly MOBILE_BREAKPOINT = 768;

  ngOnInit() {
    this.checkScreenSize();
  }

  ngOnDestroy() {
    // Cleanup if needed
  }

  @HostListener('window:resize')
  onResize() {
    this.checkScreenSize();
  }

  private checkScreenSize() {
    const wasMobile = this.isMobile;
    this.isMobile = window.innerWidth <= this.MOBILE_BREAKPOINT;
    
    // If switching to mobile, close drawer by default
    if (!wasMobile && this.isMobile) {
      this.isDrawerOpen = false;
    }
    // If switching to desktop, open drawer by default
    else if (wasMobile && !this.isMobile) {
      this.isDrawerOpen = true;
    }
  }

  toggleDrawer() {
    this.isDrawerOpen = !this.isDrawerOpen;
  }

  onDrawerClosed() {
    this.isDrawerOpen = false;
  }

  async sendRequest() {

  }

  setPage(page: string) {
    this.page = page;
    // Close drawer on mobile after navigation
    if (this.isMobile) {
      this.isDrawerOpen = false;
    }
  }

  getProfileUrl()  : string {
    return 'place_holder.webp';
  }

}
