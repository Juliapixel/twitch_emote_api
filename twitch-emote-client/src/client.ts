import { ChatUserstate, Client as TmiClient } from "tmi-js";

export interface ClientConfig {
    channels: string[];
    maxEmotesPerMessage: number;
    emotesApi: string;
}

const defaultConfig: ClientConfig = {
    channels: [],
    maxEmotesPerMessage: 5,
    emotesApi: "https://overlay-api.juliapixel.com"
};

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
    }
}

export type CallbackEmoteInfo = ChannelEmote & {source: string};

export type EmoteCallback = (emotes: CallbackEmoteInfo[], channelMessage: string) => void;

export class EmotesClient {
    public config: ClientConfig;
    private emoteCache: Map<string, Map<string, ChannelEmote>> = new Map();
    private listeners: Map<string, EmoteCallback[]> = new Map();
    private refreshInterval: number;
    private chatClient: TmiClient;

    constructor(config: Partial<ClientConfig>) {
        this.config = Object.assign(defaultConfig, config);
        this.config.channels.map((c) => c.toLowerCase())

        let tmiClient = new TmiClient({
            channels: window.structuredClone(config.channels)
        });
        tmiClient.on("message", this.handleMessage.bind(this));
        tmiClient.connect();
        this.chatClient = tmiClient;

        this.config.channels.forEach((c) => this.updateChannelEmotes(c));
        this.updateGlobalEmotes();

        this.refreshInterval = window.setInterval(
            () => {
                this.config.channels.forEach((c) => this.updateChannelEmotes(c));
            },
            1000 * 60 * 15
        );
    }

    /** please call this before dropping xqcL */
    close() {
        this.chatClient.disconnect();
        clearInterval(this.refreshInterval);
    }

    handleMessage(
        channel: string,
        _state: ChatUserstate,
        message: string,
        _self: boolean
    ) {
        let channelEmotes = this.emoteCache.get(channel.substring(1));
        let globalEmotes = this.emoteCache.get("global");
        let emotes: CallbackEmoteInfo[] = [];
        if (channelEmotes !== undefined && globalEmotes !== undefined) {
            for (const word of message.split(" ")) {
                let channelEmote = channelEmotes.get(word);
                let globalEmote = globalEmotes.get(word);
                if (channelEmote) {
                    (channelEmote as CallbackEmoteInfo).source = channel;
                    emotes.push(channelEmote as CallbackEmoteInfo)
                } else if (globalEmote) {
                    (globalEmote as CallbackEmoteInfo).source = "globals";
                    emotes.push(globalEmote as CallbackEmoteInfo)
                }
            }
        }
        if (emotes.length > 0) {
            const handlers = this.listeners.get("emote");
            for (const handler of handlers ? handlers : []) {
                handler(emotes, channel.substring(1));
            }
        }
    }

    async updateChannelEmotes(channel: string): Promise<void> {
        let resp: Record<string, ChannelEmote> = await (
            await fetch(this.config.emotesApi + "/user/" + channel)
        ).json();
        this.emoteCache.set(channel, new Map(Object.entries(resp)));
    }

    async updateGlobalEmotes() {
        let globals: Map<string, ChannelEmote> = new Map();
        for (const platform of ["7tv", "bttv", "ffz"]) {
            let resp: Record<string, ChannelEmote> = await (
                await fetch(this.config.emotesApi + "/emote/globals/" + platform)
            ).json();

            Object.entries(resp).forEach(([name, emoteInfo]) => {
                globals.set(name, emoteInfo)
            })

        }
        this.emoteCache.set("global", globals)
    }

    on(event: "emote", callback: EmoteCallback) {
        let listeners = this.listeners.get("emote");
        if (!listeners) {
            this.listeners.set("emote", [callback]);
        } else {
            listeners.push(callback);
        }
    }
}
