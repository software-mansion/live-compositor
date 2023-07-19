import express, { Express, Request, Response } from 'express';
import { Resolution, SessionId, Url } from './common';
import { Session } from './session';
import { constants as HttpConstants } from 'http2';
import { randomUUID } from 'crypto';

export class Server {
    private server: Express;
    private sessions: Map<SessionId, Session>;

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
        this.server.post("/new_session", this.new_session.bind(this));
        this.server.post("/get_frame", this.get_frame.bind(this));
    }

    private new_session(req: Request<object, object, NewSessionRequest>, res: Response<NewSessionResponse>): void {
        const data = req.body;
        console.log(`Starting rendering for ${data.url}`)

        const session_id = randomUUID();
        const session = new Session(data.url, data.resolution);
        session.run();
        this.sessions.set(session_id, session);

        res
            .status(HttpConstants.HTTP_STATUS_CREATED)
            .send({
                session_id: session_id
            });
    }

    private get_frame(req: Request<object, object, RenderRequest>, res: Response<RenderResponse>): void {
        const data = req.body;
        if (!this.sessions.has(data.session_id)) {
            res
                .status(HttpConstants.HTTP_STATUS_NOT_FOUND)
                .send({
                    error: "Session does not exist"
                });
            return;
        }

        const session = this.sessions.get(data.session_id)
        res.status(HttpConstants.HTTP_STATUS_OK).send(session.frame);
    }
}

interface NewSessionRequest {
    url: Url,
    resolution: Resolution,
}

interface NewSessionResponse {
    session_id: SessionId
}


interface RenderRequest {
    session_id: SessionId,
}

type RenderResponse = Buffer | ErrorResponse;


interface ErrorResponse {
    error: string
}
