import { Texture, Vector2 } from "three";
export interface EmoteTexture extends Texture {
    atlasInfo?: AtlasTextureInfo;
    aspectRatio: number;
}
/** information for a texture atlas containing every animation frame of an emote */
export declare class AtlasTextureInfo {
    x_size: number;
    y_size: number;
    private delays;
    private animationLength;
    constructor(x_size: number, y_size: number, delays: number[]);
    animate(timestamp: number): Vector2[];
}
//# sourceMappingURL=texture.d.ts.map