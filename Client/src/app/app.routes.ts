import { Routes } from '@angular/router';
import {Login} from './pages/login/login';
import {Main} from './pages/main/main';
import {List} from './pages/list/list';

export const routes: Routes = [
  { path: '', redirectTo: 'login', pathMatch: 'full'},
  { path: 'login', component: Login },
  { path: 'app', component: Main },
  { path: 'list', component: List },
];
