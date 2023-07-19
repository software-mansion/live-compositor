import { app } from 'electron';
import { Server } from './server';

function main(): void {
    if (process.argv.length != 3) {
        console.error("<port> not provided");
        process.exit(1);
    }

    const port = parseInt(process.argv[2]);
    const server = new Server();
    server.listen(port);
}

app.whenReady().then(main);
app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') {
        app.quit();
    }
});
