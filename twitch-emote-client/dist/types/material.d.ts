import { MeshBasicMaterial, Vector2 } from "three";
import { ChannelEmote } from "./client.js";
export declare class EmoteMaterial extends MeshBasicMaterial {
    private animationLength;
    private currentFrame;
    aspectRatio: number;
    isAnimated: boolean;
    private atlasTex?;
    constructor(channel: string, emote: ChannelEmote, apiUrl: string, onLoad?: (mat: EmoteMaterial) => void | Promise<void>);
    animateTexture(timestamp: number): Vector2[] | undefined;
}
//# sourceMappingURL=material.d.ts.map