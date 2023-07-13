import express, { Express, Request, Response } from 'express';
import { Resolution, Url } from './common';
import { Session } from './session';

export class Server {
    private server: Express;
    private sessions: Map<Url, Session>;

    public constructor() {
        this.server = express();
        this.server.use(express.json());
        this.sessions = new Map();

        this.initRoutes();
    }

    public listen(port: number): void {
        this.server.listen(port, () => {
            console.log(`Listening on ${port}`);
        });
    }

    private initRoutes(): void {
        this.server.post("/render", this.render.bind(this));
    }

    private render(req: Request<{}, {}, RenderRequest>, res: Response<Buffer>): void {
        const data = req.body;
        let session: Session;

        if (this.sessions.has(data.url)) {
            session = this.sessions.get(data.url);
        } else {
            console.log(`Starting rendering for ${data.url}`)
            session = new Session(data.url, data.resolution);
            session.run();
            this.sessions.set(data.url, session);
        }

        if (session.resolution.width != data.resolution.width ||
            session.resolution.height != data.resolution.height) {
            session.resize(data.resolution);
        }

        res.send(session.frame);
    }
}

interface RenderRequest {
    url: Url,
    resolution: Resolution,
}

interface RenderResponse {
    frame: number[]
}
