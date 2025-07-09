import { Routes } from '@angular/router';
import {Login} from './pages/login/login';
import {Main} from './pages/main/main';

export const routes: Routes = [
  { path: 'login', component: Login },
  { path: 'app', component: Main}
];
