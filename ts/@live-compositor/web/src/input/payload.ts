export type VideoPayload = { type: 'chunk'; chunk: EncodedVideoChunk } | { type: 'eos' };
