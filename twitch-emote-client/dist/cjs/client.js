import { Client as TmiClient } from "tmi-js";
const defaultConfig = {
    channels: [],
    maxEmotesPerMessage: 5,
    emotesApi: "https://overlay-api.juliapixel.com"
};
export class EmotesClient {
    constructor(config) {
        this.emoteCache = new Map();
        this.listeners = new Map();
        this.config = Object.assign(defaultConfig, config);
        let tmiClient = new TmiClient({
            channels: window.structuredClone(config.channels)
        });
        tmiClient.on("message", this.handleMessage.bind(this));
        tmiClient.connect();
        this.config.channels.forEach((c) => this.updateChannelEmotes(c));
        this.refreshInterval = setInterval(() => {
            this.config.channels.forEach((c) => this.updateChannelEmotes(c));
        }, 1000 * 60 * 15);
    }
    close() {
        clearInterval(this.refreshInterval);
    }
    handleMessage(channel, state, message, self) {
        let channelEmotes = this.emoteCache.get(channel.substring(1));
        let emotes = [];
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
    async updateChannelEmotes(channel) {
        let resp = await (await fetch(this.config.emotesApi + "/user/" + channel)).json();
        this.emoteCache.set(channel, new Map(Object.entries(resp)));
    }
    on(event, callback) {
        let listeners = this.listeners.get("emote");
        if (!listeners) {
            this.listeners.set("emote", [callback]);
        }
        else {
            listeners.push(callback);
        }
    }
}
//# sourceMappingURL=client.js.map