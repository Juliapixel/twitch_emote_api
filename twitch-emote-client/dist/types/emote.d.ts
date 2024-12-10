import { Mesh } from "three";
import { EmoteBasicMaterial, EmoteStandardMaterial, MaterialKind } from "./material.js";
import { ChannelEmote } from "./client.js";
type OnLoadHandler = (emote: EmoteObject) => Promise<any> | any;
export declare class EmoteObject extends Mesh {
    material: EmoteBasicMaterial | EmoteStandardMaterial;
    constructor(channel: string, apiUrl: string, emoteInfo: ChannelEmote, 
    /** @default MaterialKind.Basic */
    materialKind?: MaterialKind, onLoad?: OnLoadHandler);
    animateTexture(timestamp: number): void;
}
export {};
//# sourceMappingURL=emote.d.ts.map