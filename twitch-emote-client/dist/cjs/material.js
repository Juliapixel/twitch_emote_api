import { LoadingManager, MeshBasicMaterial, TextureLoader } from "three";
let cache = new Map();
export class EmoteMaterial extends MeshBasicMaterial {
    constructor(channel, emote, apiUrl, onLoad) {
        super({ transparent: true, side: 2 });
        this.frames = [];
        this.animationLength = 0;
        this.currentFrame = 0;
        this.aspectRatio = 1;
        let hit = cache.get(`channel:${channel},emote:${emote.name}`);
        if (hit) {
            Object.assign(this, hit);
            this.map = this.frames[0].texture;
            if (onLoad) {
                onLoad(this);
            }
            return;
        }
        let urlPrefix = (channel == "globals" || channel == "global") ?
            `${apiUrl}/emote/globals/${emote.platform}` :
            `${apiUrl}/emote/${channel.replace(/^\#/, "")}`;
        fetch(`${urlPrefix}/${emote.name}`).then(async (resp) => {
            let emoteInfo = await resp.json();
            this.animationLength = emoteInfo.frame_delays.reduce((sum, delay) => (sum += delay));
            this.currentFrame = 0;
            this.frames = [];
            let processedFrameCount = 0;
            for (let i = 0; i < emoteInfo.frame_count; i++) {
                let textureLoader = new TextureLoader(new LoadingManager());
                textureLoader
                    .loadAsync(`${urlPrefix}/${emote.name}/${i}.webp`)
                    .then((tex) => {
                    if (i === 0) {
                        this.map = tex;
                        this.aspectRatio =
                            tex.source.data.naturalWidth /
                                tex.source.data.naturalHeight;
                    }
                    tex.colorSpace = "srgb";
                    this.frames[i] = {
                        texture: tex,
                        delay: emoteInfo.frame_delays[i]
                    };
                    processedFrameCount += 1;
                    if (processedFrameCount === emoteInfo.frame_count) {
                        cache.set(`channel:${channel},emote:${emote.name}`, {
                            frames: this.frames,
                            aspectRatio: this.aspectRatio,
                            animationLength: this.animationLength
                        });
                        if (onLoad) {
                            onLoad(this);
                        }
                    }
                });
            }
        });
    }
    dispose() {
        for (const tex of this.frames) {
            tex.texture.dispose();
        }
        super.dispose();
    }
    animateTexture(timestamp) {
        let currentDelay = timestamp % this.animationLength;
        let delaySum = 0;
        let i = 0;
        for (const frame of this.frames) {
            if (currentDelay > delaySum) {
                this.map = frame.texture;
                this.currentFrame = i;
            }
            delaySum += frame.delay;
            i++;
        }
    }
}
//# sourceMappingURL=material.js.map