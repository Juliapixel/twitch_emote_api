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
    name: string;
}

type EmoteCallback = (emotes: ChannelEmote[], channel: string) => void;

export class EmotesClient {
    public config: ClientConfig;
    private emoteCache: Map<string, Map<string, ChannelEmote>> = new Map();
    private listeners: Map<string, EmoteCallback[]> = new Map();
    private refreshInterval: number;

    constructor(config: Partial<ClientConfig>) {
        this.config = Object.assign(defaultConfig, config);

        let tmiClient = new TmiClient({
            channels: window.structuredClone(config.channels)
        });
        tmiClient.on("message", this.handleMessage.bind(this));
        tmiClient.connect();

        this.config.channels.forEach((c) => this.updateChannelEmotes(c));
        this.refreshInterval = setInterval(
            () => {
                this.config.channels.forEach((c) => this.updateChannelEmotes(c));
            },
            1000 * 60 * 15
        );
    }

    close() {
        clearInterval(this.refreshInterval);
    }

    handleMessage(
        channel: string,
        state: ChatUserstate,
        message: string,
        self: boolean
    ) {
        let channelEmotes = this.emoteCache.get(channel.substring(1));
        let emotes: ChannelEmote[] = [];
        if (channelEmotes !== undefined) {
            for (const word of message.split(" ")) {
                let emote = channelEmotes.get(word);
                if (emote) {
                    emotes.push(emote);
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
        let resp: Object = await (
            await fetch(this.config.emotesApi + "/user/" + channel)
        ).json();
        this.emoteCache.set(channel, new Map(Object.entries(resp)));
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
