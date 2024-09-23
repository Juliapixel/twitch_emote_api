import { Mesh, PlaneGeometry } from "three";
import { EmoteMaterial } from "./material.js";
export class EmoteObject extends Mesh {
    material;
    constructor(channel, apiUrl, emoteInfo, onLoad) {
        let geometry = new PlaneGeometry();
        super(geometry);
        this.material = new EmoteMaterial(channel, emoteInfo, apiUrl, (mat) => {
            this.scale.x = mat.aspectRatio;
            onLoad ? onLoad(this) : {};
        });
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