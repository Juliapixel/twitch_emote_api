import { MeshBasicMaterial, MeshStandardMaterial } from "three";
/** kind of material used for the emote object */
export var MaterialKind;
(function (MaterialKind) {
    /** MeshBasicMaterial */
    MaterialKind[MaterialKind["Basic"] = 0] = "Basic";
    /** MeshStandardMaterial */
    MaterialKind[MaterialKind["Standard"] = 1] = "Standard";
})(MaterialKind || (MaterialKind = {}));
export class EmoteBasicMaterial extends MeshBasicMaterial {
    constructor(params) {
        super({ transparent: true, map: params.map });
    }
    animateTexture(timestamp) {
        if (this.map.atlasInfo !== undefined) {
            return this.map.atlasInfo.animate(timestamp);
        }
        return undefined;
    }
}
export class EmoteStandardMaterial extends MeshStandardMaterial {
    constructor(params) {
        super({ transparent: true, map: params.map });
    }
    animateTexture(timestamp) {
        if (this.map.atlasInfo !== undefined) {
            return this.map.atlasInfo.animate(timestamp);
        }
        return undefined;
    }
}
//# sourceMappingURL=material.js.map