import { z } from 'zod';

export type Resolution = z.infer<typeof ResolutionScheme>;

const ResolutionScheme = z.object({
    width: z.number().positive(),
    height: z.number().positive()
});

export const NewSessionRequest = z.object({
    url: z.string().url(),
    resolution: ResolutionScheme,
});


export const GetFrameRequest = z.object({
    session_id: z.string().uuid()
});
