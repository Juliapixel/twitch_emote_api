import { Mesh, PlaneGeometry } from "three";
import { EmoteMaterial } from "./material.js";
export class EmoteObject extends Mesh {
    constructor(channel, apiUrl, emoteInfo, onLoad) {
        let geometry = new PlaneGeometry();
        super(geometry);
        this.material = new EmoteMaterial(channel, emoteInfo, apiUrl, (mat) => {
            this.scale.x = mat.aspectRatio;
            onLoad ? onLoad(this) : {};
        });
    }
    animateTexture(timestamp) {
        this.material.animateTexture(timestamp);
    }
}
//# sourceMappingURL=emote.js.map