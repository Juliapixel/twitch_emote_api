import { Mesh, PlaneGeometry } from "three";
import { EmoteMaterial } from "./material.js";
import { ChannelEmote } from "./client.js";

type OnLoadHandler = (emote: EmoteObject) => Promise<any> | any;

export class EmoteObject extends Mesh {
    material: EmoteMaterial;

    constructor(
        channel: string,
        apiUrl: string,
        emoteInfo: ChannelEmote,
        onLoad?: OnLoadHandler
    ) {
        let geometry = new PlaneGeometry();
        super(geometry);
        this.material = new EmoteMaterial(channel, emoteInfo, apiUrl, (mat) => {
            this.material = mat;
            this.scale.x = mat.aspectRatio;
            onLoad ? onLoad(this) : {};
        });
    }

    animateTexture(timestamp: number) {
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
