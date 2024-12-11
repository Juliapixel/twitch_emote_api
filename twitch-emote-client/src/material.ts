import {
    MeshBasicMaterial,
    MeshBasicMaterialParameters,
    MeshStandardMaterial,
    MeshStandardMaterialParameters,
    Vector2
} from "three";
import { EmoteTexture } from "./texture.js";

/** kind of material used for the emote object */
export enum MaterialKind {
    /** MeshBasicMaterial */
    Basic,
    /** MeshStandardMaterial */
    Standard
}

interface IEmoteMaterial {
    map: EmoteTexture;

    animateTexture(timestamp: number): Vector2[] | undefined;
}

export class EmoteBasicMaterial extends MeshBasicMaterial implements IEmoteMaterial {
    declare map: EmoteTexture;

    constructor(params: MeshBasicMaterialParameters & { map: EmoteTexture }) {
        super({ transparent: true, map: params.map });
    }

    animateTexture(timestamp: number): Vector2[] | undefined {
        if (this.map.atlasInfo !== undefined) {
            return this.map.atlasInfo.animate(timestamp);
        }
        return undefined;
    }
}

export class EmoteStandardMaterial
    extends MeshStandardMaterial
    implements IEmoteMaterial
{
    declare map: EmoteTexture;

    constructor(params: MeshStandardMaterialParameters & { map: EmoteTexture }) {
        super({ transparent: true, map: params.map });
    }

    animateTexture(timestamp: number): Vector2[] | undefined {
        if (this.map.atlasInfo !== undefined) {
            return this.map.atlasInfo.animate(timestamp);
        }
        return undefined;
    }
}
