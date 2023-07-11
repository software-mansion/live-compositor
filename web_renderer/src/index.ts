import { app, BrowserWindow } from 'electron';
import { Server } from './server';

function main(): void {
  const server = new Server(8080);
  server.listen();
}

app.on('ready', main);

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});


