import { MeshBasicMaterial, MeshBasicMaterialParameters, MeshStandardMaterial, MeshStandardMaterialParameters, Vector2 } from "three";
import { EmoteTexture } from "./texture.js";
/** kind of material used for the emote object */
export declare enum MaterialKind {
    /** MeshBasicMaterial */
    Basic = 0,
    /** MeshStandardMaterial */
    Standard = 1
}
interface IEmoteMaterial {
    map: EmoteTexture;
    animateTexture(timestamp: number): Vector2[] | undefined;
}
export declare class EmoteBasicMaterial extends MeshBasicMaterial implements IEmoteMaterial {
    map: EmoteTexture;
    constructor(params: MeshBasicMaterialParameters & {
        map: EmoteTexture;
    });
    animateTexture(timestamp: number): Vector2[] | undefined;
}
export declare class EmoteStandardMaterial extends MeshStandardMaterial implements IEmoteMaterial {
    map: EmoteTexture;
    constructor(params: MeshStandardMaterialParameters & {
        map: EmoteTexture;
    });
    animateTexture(timestamp: number): Vector2[] | undefined;
}
export {};
//# sourceMappingURL=material.d.ts.map