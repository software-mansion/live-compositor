import { app } from 'electron';
import { Server } from './server';

function main(): void {
    const port = process.env.WEB_RENDERER_PORT;
    if (port == null) {
        console.error("env WEB_RENDERER_PORT not defined");
        process.exit(1);
    }

    const server = new Server();
    server.listen(parseInt(port));
}

app.whenReady().then(main);
app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') {
        app.quit();
    }
});
