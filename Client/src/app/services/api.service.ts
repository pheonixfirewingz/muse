import { Injectable } from '@angular/core';
import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Observable, throwError, of } from 'rxjs';
import { catchError, switchMap } from 'rxjs/operators';

export interface ApiConfig {
  host: string;
  port: number;
  useHttps: boolean;
}

@Injectable({
  providedIn: 'root'
})
export class ApiService {
  private config: ApiConfig = {
    host: '127.0.0.1',
    port: 8000,
    useHttps: true // Try HTTPS first, fallback to HTTP if needed
  };

  private baseUrl: string = '';

  constructor(private http: HttpClient) {
    this.updateBaseUrl();
  }

  private updateBaseUrl(): void {
    const protocol = this.config.useHttps ? 'https' : 'http';
    this.baseUrl = `${protocol}://${this.config.host}:${this.config.port}`;
  }

  /**
   * Configure the API service with custom settings
   */
  configure(config: Partial<ApiConfig>): void {
    this.config = { ...this.config, ...config };
    this.updateBaseUrl();
  }

  /**
   * Get the current base URL
   */
  getBaseUrl(): string {
    return this.baseUrl;
  }

  /**
   * Make an API request with automatic HTTPS/HTTP fallback
   */
  request<T>(method: string, endpoint: string, data?: any): Observable<T> {
    return this.http.request<T>(method, `${this.baseUrl}${endpoint}`, {
      body: data
    }).pipe(
      catchError((error: HttpErrorResponse) => {
        // If HTTPS fails and we haven't tried HTTP yet, fall back to HTTP
        if (error.status === 0 && this.config.useHttps) {
          console.log('HTTPS failed, falling back to HTTP...');
          this.config.useHttps = false;
          this.updateBaseUrl();
          
          // Retry the request with HTTP
          return this.http.request<T>(method, `${this.baseUrl}${endpoint}`, {
            body: data
          });
        }
        
        return throwError(() => error);
      })
    );
  }

  /**
   * GET request
   */
  get<T>(endpoint: string): Observable<T> {
    return this.request<T>('GET', endpoint);
  }

  /**
   * POST request
   */
  post<T>(endpoint: string, data?: any): Observable<T> {
    return this.request<T>('POST', endpoint, data);
  }

  /**
   * PUT request
   */
  put<T>(endpoint: string, data?: any): Observable<T> {
    return this.request<T>('PUT', endpoint, data);
  }

  /**
   * DELETE request
   */
  delete<T>(endpoint: string): Observable<T> {
    return this.request<T>('DELETE', endpoint);
  }

  /**
   * Test connection to determine available protocol
   */
  testConnection(): Observable<{ https: boolean; http: boolean }> {
    const results = { https: false, http: false };

    // Always use HTTP for health check to avoid SSL errors
    const httpUrl = `http://${this.config.host}:${this.config.port}/api/health`;
    const healthCheck = this.http.get(httpUrl).pipe(
      catchError(() => of(null)),
      switchMap((response: any) => {
        if (response && response.protocols) {
          results.http = response.protocols.http || false;
          results.https = response.protocols.https || false;
        }
        return of(results);
      })
    );

    return healthCheck;
  }
} 