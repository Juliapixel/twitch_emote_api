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
export type CallbackEmoteInfo = ChannelEmote & {
    source: string;
};
export type EmoteCallback = (emotes: CallbackEmoteInfo[], channelMessage: string) => void;
export declare class EmotesClient {
    config: ClientConfig;
    private emoteCache;
    private listeners;
    private refreshInterval;
    private chatClient;
    constructor(config: Partial<ClientConfig>);
    /** please call this before dropping xqcL */
    close(): void;
    handleMessage(channel: string, _state: ChatUserstate, message: string, _self: boolean): void;
    updateChannelEmotes(channel: string): Promise<void>;
    updateGlobalEmotes(): Promise<void>;
    on(event: "emote", callback: EmoteCallback): void;
}
//# sourceMappingURL=client.d.ts.map