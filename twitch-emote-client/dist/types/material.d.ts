import { MeshBasicMaterial } from "three";
import { ChannelEmote } from "./client.js";
export declare class EmoteMaterial extends MeshBasicMaterial {
    private frames;
    private animationLength;
    private currentFrame;
    aspectRatio: number;
    constructor(channel: string, emote: ChannelEmote, apiUrl: string, onLoad?: (mat: EmoteMaterial) => void | Promise<void>);
    dispose(): void;
    animateTexture(timestamp: number): void;
}
//# sourceMappingURL=material.d.ts.map