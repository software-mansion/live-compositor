import { Url } from "./common";
import { Packet } from "./packet";

export enum CommandType {
    use,
    resolution,
    source,
    render,
    unknown
}

export type Command =
    { type: CommandType.use, url: Url }
    | { type: CommandType.resolution, width: number, height: number }
    | { type: CommandType.source, data: Buffer }
    | { type: CommandType.render }
    | { type: CommandType.unknown };

export function getCommand(packet: Packet): Command {
    const data = packet.toString();
    const commandSepIdx = data.indexOf(":");
    const commandName = commandSepIdx == -1 ? data : data.substring(0, commandSepIdx);
    const command = CommandType[commandName as keyof typeof CommandType];
    const arg = data.substring(commandSepIdx + 1);

    switch (command) {
        case CommandType.use:
            return { type: command, url: arg };
        case CommandType.resolution:
            const resolution = arg.split("x");
            return { type: command, width: parseInt(resolution[0]), height: parseInt(resolution[1]) };
        case CommandType.source:
            console.warn("Unimplemented");
            return { type: CommandType.unknown };
        case CommandType.render:
            return { type: CommandType.render }
        default:
            console.warn(`Unknown command: ${commandName}`);
            return { type: CommandType.unknown };
    }
}
