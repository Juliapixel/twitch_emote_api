import { Client as TmiClient } from "tmi-js";
const defaultConfig = {
    channels: [],
    maxEmotesPerMessage: 5,
    emotesApi: "https://overlay-api.juliapixel.com"
};
export class EmotesClient {
    config;
    emoteCache = new Map();
    listeners = new Map();
    refreshInterval;
    chatClient;
    constructor(config) {
        this.config = Object.assign(defaultConfig, config);
        this.config.channels.map((c) => c.toLowerCase());
        let tmiClient = new TmiClient({
            channels: window.structuredClone(config.channels)
        });
        tmiClient.on("message", this.handleMessage.bind(this));
        tmiClient.connect();
        this.chatClient = tmiClient;
        this.config.channels.forEach((c) => this.updateChannelEmotes(c));
        this.updateGlobalEmotes();
        this.refreshInterval = window.setInterval(() => {
            this.config.channels.forEach((c) => this.updateChannelEmotes(c));
        }, 1000 * 60 * 15);
    }
    /** please call this before dropping xqcL */
    close() {
        this.chatClient.disconnect();
        clearInterval(this.refreshInterval);
    }
    async handleMessage(channel, state, message, _self) {
        let channelEmotes = this.emoteCache.get(channel.substring(1));
        let globalEmotes = this.emoteCache.get("global");
        let emotes = [];
        for (const twitchEmoteId of Object.entries(state.emotes ?? {})) {
            let info = await (await fetch(this.config.emotesApi + `/emote/twitch/${twitchEmoteId[0]}`)).json();
            info.source = "twitch_emote";
            for (let i = 0; i < twitchEmoteId[1].length; i++) {
                emotes.push(info);
            }
        }
        if (channelEmotes !== undefined && globalEmotes !== undefined) {
            for (const word of message.split(" ")) {
                let channelEmote = channelEmotes.get(word);
                let globalEmote = globalEmotes.get(word);
                if (channelEmote) {
                    channelEmote.source = channel;
                    emotes.push(channelEmote);
                }
                else if (globalEmote) {
                    globalEmote.source = "globals";
                    emotes.push(globalEmote);
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
    async updateGlobalEmotes() {
        let globals = new Map();
        for (const platform of ["7tv", "bttv", "ffz"]) {
            let resp = await (await fetch(this.config.emotesApi + "/emote/globals/" + platform)).json();
            Object.entries(resp).forEach(([name, emoteInfo]) => {
                globals.set(name, emoteInfo);
            });
        }
        this.emoteCache.set("global", globals);
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