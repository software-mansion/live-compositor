import { app } from 'electron';
import { startServer } from './server';

function main(): void {
    if (process.argv.length != 3) {
        console.error("<port> not provided");
        process.exit(1);
    }

    const port = parseInt(process.argv[2]);
    startServer(port);
}

app.whenReady().then(main);
app.on('window-all-closed', () => {
    app.quit();
});
