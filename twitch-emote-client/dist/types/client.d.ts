import { ChatUserstate } from "tmi-js";
export interface ClientConfig {
    channels: string[];
    maxEmotesPerMessage: number;
    emotesApi: string;
}
export interface ChannelEmote {
    platform: string;
    id: string;
    width: number;
    height: number;
    name: string;
    animated: boolean;
    frame_count: number;
    frame_delays: number[];
    atlas_info?: {
        x_size: number;
        y_size: number;
    };
}
export type CallbackEmoteInfo = ChannelEmote & {
    source: string;
};
export type EmoteCallback = (emotes: CallbackEmoteInfo[], source: string) => void;
export declare class EmotesClient {
    config: ClientConfig;
    private emoteCache;
    private listeners;
    private refreshInterval;
    private chatClient;
    constructor(config: Partial<ClientConfig>);
    /** please call this before dropping xqcL */
    close(): void;
    handleMessage(channel: string, state: ChatUserstate, message: string, _self: boolean): Promise<void>;
    updateChannelEmotes(channel: string): Promise<void>;
    updateGlobalEmotes(): Promise<void>;
    on(event: "emote", callback: EmoteCallback): void;
}
//# sourceMappingURL=client.d.ts.map