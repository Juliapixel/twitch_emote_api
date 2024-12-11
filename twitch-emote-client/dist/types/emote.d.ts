import { Mesh } from "three";
import { EmoteBasicMaterial, EmoteStandardMaterial } from "./material.js";
/**
 * Plane mesh with texture corresponding to a twitch chat emote
 */
export declare class EmoteObject extends Mesh {
    material: EmoteBasicMaterial | EmoteStandardMaterial;
    constructor(material: EmoteBasicMaterial | EmoteStandardMaterial);
    animateTexture(timestamp: number): void;
}
//# sourceMappingURL=emote.d.ts.map