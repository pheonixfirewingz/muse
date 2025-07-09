import axios from 'axios';
import { Router } from '@angular/router';

export function setupAxiosInterceptor(router: Router) {
  axios.interceptors.response.use(
    response => response,
    error => {
      if (error.response && error.response.status === 401) {
        router.navigate(['login']);
      }
      return Promise.reject(error);
    }
  );
} 