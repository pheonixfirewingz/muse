import { Component, OnInit, OnDestroy, HostListener, ViewChild } from '@angular/core';
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
import { MusicPlayerService, SongSearchResult } from '../../services/music-player.service';
import { debounceTime, Subject } from 'rxjs';
import { ArtistDetail } from './artists/artist-detail/artist-detail';
import {Profile} from './profile/profile';

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
    ArtistDetail,
    Profile,
    // Add ArtistDetail here
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

  searchResults: SongSearchResult[] = [];
  showDropdown = false;
  private searchInput$ = new Subject<string>();
  private searchInputSub: any;

  selectedArtistName: string = '';

  @ViewChild(Songs) songsComponent?: Songs;
  @ViewChild(Artists) artistsComponent?: Artists;
  constructor(private musicPlayerService: MusicPlayerService) {}

  ngOnInit() {
    this.checkScreenSize();
    this.searchInputSub = this.searchInput$.pipe(debounceTime(200)).subscribe(async (value) => {
      await this.handleSearchInput(value);
    });
    this.search.get('query')?.valueChanges?.subscribe((value) => {
      this.searchInput$.next(value || '');
    });
  }

  ngOnDestroy() {
    if (this.searchInputSub) this.searchInputSub.unsubscribe();
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
    const query = this.search.get('query')?.value?.trim() || '';
    let searchTerm = '';
    if (query.startsWith('!s ')) {
      searchTerm = query.slice(3).trim();
    } else if (query.startsWith('!')) {
      // Future: handle other !* commands here
      return;
    } else {
      searchTerm = query;
    }
    if (searchTerm.length === 0) return;
    try {
      const results: SongSearchResult[] = await this.musicPlayerService.fuzzySearchSongs(searchTerm);
      if (this.songsComponent) {
        this.songsComponent.setSongs(results);
        this.setPage('songs');
      }
    } catch (e) {
      alert('Song search failed.');
    }
  }

  async handleSearchInput(value: string) {
    const query = value.trim();
    let searchTerm = '';
    if (query.startsWith('!s ')) {
      searchTerm = query.slice(3).trim();
    } else if (query.startsWith('!')) {
      // Future: handle other !* commands here
      this.searchResults = [];
      this.showDropdown = false;
      return;
    } else {
      searchTerm = query;
    }
    if (searchTerm.length === 0) {
      this.searchResults = [];
      this.showDropdown = false;
      return;
    }
    try {
      const results: SongSearchResult[] = await this.musicPlayerService.fuzzySearchSongs(searchTerm);
      this.searchResults = results;
      this.showDropdown = results.length > 0;
    } catch (e) {
      this.searchResults = [];
      this.showDropdown = false;
    }
  }

  onDropdownSelect(song: SongSearchResult) {
    this.musicPlayerService.playSong(song.name, song.artist_name);
    this.showDropdown = false;
    this.search.get('query')?.setValue('');
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

  showArtistDetail(artistName: string) {
    this.selectedArtistName = artistName;
    this.setPage('artist-detail');
  }

  backToArtists() {
    this.selectedArtistName = '';
    this.setPage('artists');
  }
}
