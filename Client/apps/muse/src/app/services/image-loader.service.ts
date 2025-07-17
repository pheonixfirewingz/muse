import { Injectable } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class ImageLoaderService {
  private worker: Worker;
  private callbacks: Map<string, (result: any) => void> = new Map();

  constructor() {
    if (typeof window !== 'undefined' && typeof Worker !== 'undefined') {
      this.worker = new Worker(new URL('./image-loader.worker', import.meta.url), { type: 'module' });
      this.worker.onmessage = (event: MessageEvent) => {
        const { url, ...result } = event.data;
        const cb = this.callbacks.get(url);
        if (cb) {
          cb(result);
          this.callbacks.delete(url);
        }
      };
    } else {
      throw new Error('Web Workers are not supported in this environment.');
    }
  }

  /**
   * Request an image from the worker. Returns a Promise that resolves with the worker's response.
   * @param url The image URL
   * @param headers Optional headers (e.g., Authorization)
   */
  requestImage(url: string, headers?: Record<string, string>): Promise<any> {
    return new Promise((resolve) => {
      this.callbacks.set(url, (result) => {
        resolve(result);
      });
      this.worker.postMessage({ url, headers });
    });
  }
} 