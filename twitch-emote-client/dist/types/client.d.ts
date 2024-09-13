import { ChatUserstate } from "tmi-js";
export interface ClientConfig {
    channels: string[];
    maxEmotesPerMessage: number;
    emotesApi: string;
}
export interface ChannelEmote {
    platform: string;
    id: string;
    name: string;
}
type EmoteCallback = (emotes: ChannelEmote[], channel: string) => void;
export declare class EmotesClient {
    config: ClientConfig;
    private emoteCache;
    private listeners;
    private refreshInterval;
    constructor(config: Partial<ClientConfig>);
    close(): void;
    handleMessage(channel: string, state: ChatUserstate, message: string, self: boolean): void;
    updateChannelEmotes(channel: string): Promise<void>;
    on(event: "emote", callback: EmoteCallback): void;
}
export {};
//# sourceMappingURL=client.d.ts.map