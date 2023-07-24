import express, { Request, Response } from 'express';
import { SessionId } from './common';
import { Session } from './session';
import { constants as HttpConstants } from 'http2';
import { randomUUID } from 'crypto';
import { GetFrameRequest, NewSessionRequest } from './schemas';
import { sessions } from './state';

interface ErrorResponse {
    error: string
}

type NewSessionResponse =
    { session_id: SessionId }
    | ErrorResponse;

type GetFrameResponse =
    Buffer
    | ErrorResponse;

    
const app = express();

app.use(express.json());

app.post("/new_session", (req: Request, res: Response<NewSessionResponse>) => {
    try {
        const data = NewSessionRequest.parse(req.body);
        console.log(`Starting rendering for ${data.url}`);
        const session_id = randomUUID();
        const session = new Session(data.url, data.resolution);
        sessions.set(session_id, session);
        session.run();

        res
            .status(HttpConstants.HTTP_STATUS_CREATED)
            .send({
                session_id: session_id
            });
    } catch (err) {
        res
            .status(HttpConstants.HTTP_STATUS_BAD_REQUEST)
            .send({ error: err.toString() });
    }
});

app.post("/get_frame", (req: Request, res: Response<GetFrameResponse>) => {
    try {
        const data = GetFrameRequest.parse(req.body);
        if (!sessions.has(data.session_id)) {
            res
                .status(HttpConstants.HTTP_STATUS_NOT_FOUND)
                .send({
                    error: "Session does not exist"
                });
            return;
        }

        const session = sessions.get(data.session_id);
        res.status(HttpConstants.HTTP_STATUS_OK).send(session.frame);
    } catch (err) {
        res
            .status(HttpConstants.HTTP_STATUS_BAD_REQUEST)
            .send({ error: err.toString() });
    }
});

export function startServer(port: number): void {
    app.listen(port, () => {
        console.log(`Listening on ${port}`);
    });
}
