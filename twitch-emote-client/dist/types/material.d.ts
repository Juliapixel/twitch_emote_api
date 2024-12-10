import { MeshBasicMaterial, MeshStandardMaterial, Vector2 } from "three";
import { ChannelEmote } from "./client.js";
import { AtlasTexture } from "./atlas.js";
/** kind of material used for the emote object */
export declare enum MaterialKind {
    /** MeshBasicMaterial */
    Basic = 0,
    /** MeshStandardMaterial */
    Standard = 1
}
interface IEmoteMaterial {
    animationLength: number;
    aspectRatio: number;
    isAnimated: boolean;
    atlasTex?: AtlasTexture;
    animateTexture(timestamp: number): Vector2[] | undefined;
}
export declare class EmoteBasicMaterial extends MeshBasicMaterial implements IEmoteMaterial {
    animationLength: number;
    aspectRatio: number;
    isAnimated: boolean;
    atlasTex?: AtlasTexture;
    constructor(source: string, emote: ChannelEmote, apiUrl: string, onLoad?: (mat: EmoteBasicMaterial) => void | Promise<void>);
    animateTexture(timestamp: number): Vector2[] | undefined;
}
export declare class EmoteStandardMaterial extends MeshStandardMaterial implements IEmoteMaterial {
    animationLength: number;
    aspectRatio: number;
    isAnimated: boolean;
    atlasTex?: AtlasTexture;
    constructor(source: string, emote: ChannelEmote, apiUrl: string, onLoad?: (mat: EmoteStandardMaterial) => void | Promise<void>);
    animateTexture(timestamp: number): Vector2[] | undefined;
}
export {};
//# sourceMappingURL=material.d.ts.map