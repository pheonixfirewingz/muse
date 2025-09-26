import {app,BrowserWindow} from 'electron';
import path from 'path';

app.on('ready', async () => {
    const win = new BrowserWindow({});
    await win.loadFile(path.join(app.getAppPath(), '/dist-react/index.html'));
})
