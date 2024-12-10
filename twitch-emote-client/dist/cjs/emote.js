import { Mesh, PlaneGeometry } from "three";
import { EmoteBasicMaterial, EmoteStandardMaterial, MaterialKind } from "./material.js";
export class EmoteObject extends Mesh {
    constructor(channel, apiUrl, emoteInfo, 
    /** @default MaterialKind.Basic */
    materialKind, onLoad) {
        let geometry = new PlaneGeometry();
        super(geometry);
        let kind;
        if (materialKind === undefined) {
            kind = MaterialKind.Basic;
        }
        else {
            kind = materialKind;
        }
        this.name = `${channel}.${emoteInfo.name}`;
        switch (kind) {
            case MaterialKind.Basic:
                this.material = new EmoteBasicMaterial(channel, emoteInfo, apiUrl, (mat) => {
                    this.material = mat;
                    this.scale.x = mat.aspectRatio;
                    onLoad ? onLoad(this) : {};
                });
                break;
            case MaterialKind.Standard:
                this.material = new EmoteStandardMaterial(channel, emoteInfo, apiUrl, (mat) => {
                    this.material = mat;
                    this.scale.x = mat.aspectRatio;
                    onLoad ? onLoad(this) : {};
                });
                break;
        }
    }
    animateTexture(timestamp) {
        let uvs = this.material.animateTexture(timestamp);
        if (!uvs) {
            return;
        }
        let uvAttr = this.geometry.attributes.uv;
        for (let i = 0; i < 4; i++) {
            uvAttr.setXY(i, uvs[i].x, uvs[i].y);
        }
        uvAttr.needsUpdate = true;
    }
}
//# sourceMappingURL=emote.js.map