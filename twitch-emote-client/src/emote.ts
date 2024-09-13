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
            this.scale.x = mat.aspectRatio;

            onLoad ? onLoad(this) : {};
        });
    }

    animateTexture(timestamp: number) {
        this.material.animateTexture(timestamp);
    }
}
