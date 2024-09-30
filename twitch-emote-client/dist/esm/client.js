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
        // second item is the freakin start index, for sorting twitch emotes
        let emotes = [];
        for (const [twitchEmoteId, positions] of Object.entries(state.emotes ?? {})) {
            let info = await (await fetch(this.config.emotesApi + `/emote/twitch/${twitchEmoteId}`)).json();
            info.source = "twitch_emote";
            for (const position of positions) {
                emotes.push([info, parseInt(position.split("-")[0], 10)]);
            }
        }
        if (channelEmotes !== undefined && globalEmotes !== undefined) {
            let idx = 0;
            for (const word of message.split(" ")) {
                if (idx !== 0) {
                    idx += 1;
                }
                let channelEmote = channelEmotes.get(word);
                let globalEmote = globalEmotes.get(word);
                if (channelEmote) {
                    channelEmote.source = channel;
                    emotes.push([channelEmote, idx]);
                }
                else if (globalEmote) {
                    globalEmote.source = "globals";
                    emotes.push([globalEmote, idx]);
                }
                idx += word.length;
            }
        }
        if (emotes.length > 0) {
            emotes.sort((a, b) => a[1] - b[1]);
            const handlers = this.listeners.get("emote");
            for (const handler of handlers ? handlers : []) {
                handler(emotes.map((i) => i[0]), channel.substring(1));
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