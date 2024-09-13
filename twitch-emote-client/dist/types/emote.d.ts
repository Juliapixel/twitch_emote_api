import { Mesh } from "three";
import { EmoteMaterial } from "./material.js";
import { ChannelEmote } from "./client.js";
type OnLoadHandler = (emote: EmoteObject) => Promise<any> | any;
export declare class EmoteObject extends Mesh {
    material: EmoteMaterial;
    constructor(channel: string, apiUrl: string, emoteInfo: ChannelEmote, onLoad?: OnLoadHandler);
    animateTexture(timestamp: number): void;
}
export {};
//# sourceMappingURL=emote.d.ts.map