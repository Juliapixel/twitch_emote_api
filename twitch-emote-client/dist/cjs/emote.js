import { Mesh, PlaneGeometry } from "three";
/**
 * Plane mesh with texture corresponding to a twitch chat emote
 */
export class EmoteObject extends Mesh {
    constructor(material) {
        let geometry = new PlaneGeometry();
        super(geometry);
        this.material = material;
        this.scale.x = this.material.map.aspectRatio;
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