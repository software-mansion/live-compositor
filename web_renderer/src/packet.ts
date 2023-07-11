import net from 'net';

export class Packet {
    private length: number;
    private message: Buffer
    
    public constructor(message: Buffer) {
        this.length = message.length;
        this.message = message;
    }

    public send(sock: net.Socket): void {
        const header = Buffer.from([
            (this.length & 0xff000000) >> 24,
            (this.length & 0x00ff0000) >> 16,
            (this.length & 0x0000ff00) >> 8,
            (this.length & 0x000000ff)
        ]);
        
        sock.write(Buffer.concat([header, this.message]))
    }

    public toString(): string {
        return this.message.toString();
    }
}