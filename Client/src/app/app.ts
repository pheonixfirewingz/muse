import {Component, inject, OnInit} from '@angular/core';
import {RouterOutlet, Router} from '@angular/router';


export async function fetchWithAuth(input: RequestInfo | URL, init?: RequestInit, router?: Router): Promise<Response> {
  const response = await fetch(input, init);
  if (response.status === 401) {
    if (router) {
      router.navigate(['/login']);
    } else {
      window.location.href = '/login';
    }
  }
  return response;
}

@Component({
  selector: 'app-root',
  imports: [RouterOutlet],
  template: '<router-outlet></router-outlet>',
})
export class App {}
