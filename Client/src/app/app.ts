import {Component, inject, OnInit} from '@angular/core';
import {RouterOutlet, Router} from '@angular/router';
import { HttpClientModule } from '@angular/common/http';
import { setupAxiosInterceptor } from './axios-interceptor';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet],
  template: '<router-outlet></router-outlet>',
})
export class App implements OnInit {
  private router = inject(Router);

  ngOnInit() {
    setupAxiosInterceptor(this.router);
  }
}
