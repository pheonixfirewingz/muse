import { platformBrowserDynamic } from '@angular/platform-browser-dynamic';
import { appConfig } from './app/app.config';
import { App } from './app/app';

platformBrowserDynamic(App, appConfig)
  .catch((err: any) => console.error(err));
