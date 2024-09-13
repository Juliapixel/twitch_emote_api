import {
    LoadingManager,
    Material,
    MeshBasicMaterial,
    Texture,
    TextureLoader
} from "three";
import { ChannelEmote } from "./client.js";

interface AnimationFrame {
    texture: Texture;
    delay: number;
}

interface EmoteInfo {
    platform: string;
    id: string;
    name: string;
    frame_count: number;
    frame_delays: number[];
}

let cache: Map<
    string,
    { frames: AnimationFrame[]; aspectRatio: number; animationLength: number }
> = new Map();

export class EmoteMaterial extends MeshBasicMaterial {
    private frames: AnimationFrame[] = [];
    private animationLength: number = 0;
    private currentFrame: number = 0;
    public aspectRatio: number = 1;

    constructor(
        channel: string,
        emote: ChannelEmote,
        apiUrl: string,
        onLoad?: (mat: EmoteMaterial) => void | Promise<void>
    ) {
        super({ transparent: true, side: 2 });

        let hit = cache.get(`channel:${channel},emote:${emote.name}`);
        if (hit) {
            Object.assign(this, hit);
            this.map = this.frames[0].texture;
            if (onLoad) {
                onLoad(this);
            }
            return;
        }

        fetch(`${apiUrl}/emote/${channel}/${emote.name}`).then(async (resp) => {
            let emoteInfo: EmoteInfo = await resp.json();
            this.animationLength = emoteInfo.frame_delays.reduce(
                (sum, delay) => (sum += delay)
            );
            this.currentFrame = 0;
            this.frames = [];

            let processedFrameCount = 0;
            for (let i = 0; i < emoteInfo.frame_count; i++) {
                let textureLoader = new TextureLoader(new LoadingManager());

                textureLoader
                    .loadAsync(`${apiUrl}/emote/${channel}/${emote.name}/${i}.webp`)
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

    dispose(): void {
        for (const tex of this.frames) {
            tex.texture.dispose();
        }
        super.dispose();
    }

    animateTexture(timestamp: number) {
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
