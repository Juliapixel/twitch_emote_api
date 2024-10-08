import { LinearMipMapNearestFilter, LoadingManager, MeshBasicMaterial, NearestFilter, SRGBColorSpace, Texture, TextureLoader } from "three";
import { AtlasTexture } from "./atlas.js";
let cache = new Map();
const loadingManager = new LoadingManager();
export class EmoteMaterial extends MeshBasicMaterial {
    animationLength = 0;
    currentFrame = 0;
    aspectRatio = 1;
    isAnimated = false;
    atlasTex;
    constructor(source, emote, apiUrl, onLoad) {
        super({ transparent: true, side: 2, alphaTest: 0.5 });
        let hit = cache.get(`channel:${source},emote:${emote.name}`);
        if (hit) {
            Object.assign(this, hit);
            if (hit.tex instanceof Texture) {
                this.map = hit.tex;
                this.isAnimated = false;
            }
            else {
                this.map = hit.tex.map;
                this.atlasTex = new AtlasTexture(hit.tex.x_size, hit.tex.y_size, hit.tex.delays);
                this.isAnimated = true;
            }
            if (onLoad) {
                onLoad(this);
            }
            return;
        }
        let urlPrefix;
        switch (source) {
            case "globals":
            case "global":
                urlPrefix = `${apiUrl}/emote/globals/${emote.platform}`;
                break;
            case "twitch_emote":
                urlPrefix = `${apiUrl}/emote/twitch`;
                break;
            default:
                urlPrefix = `${apiUrl}/emote/${source.replace(/^\#/, "")}`;
        }
        fetch(`${urlPrefix}/${emote.name}`).then(async (resp) => {
            let emoteInfo = await resp.json();
            this.isAnimated = emoteInfo.frame_count > 1;
            this.animationLength = emoteInfo.frame_delays.reduce((sum, delay) => (sum += delay));
            this.currentFrame = 0;
            let texUrl = this.isAnimated
                ? `${urlPrefix}/${emote.name}/atlas.webp`
                : `${urlPrefix}/${emote.name}/0.webp`;
            let textureLoader = new TextureLoader(loadingManager);
            textureLoader.loadAsync(texUrl).then((tex) => {
                tex.magFilter = NearestFilter;
                tex.minFilter = LinearMipMapNearestFilter;
                this.map = tex;
                this.aspectRatio = emoteInfo.width / emoteInfo.height;
                if (this.isAnimated && emoteInfo.atlas_info) {
                    this.atlasTex = new AtlasTexture(emoteInfo.atlas_info.x_size, emoteInfo.atlas_info.y_size, emoteInfo.frame_delays);
                }
                tex.colorSpace = SRGBColorSpace;
                cache.set(`channel:${source},emote:${emote.name}`, {
                    animationLength: this.animationLength,
                    aspectRatio: this.aspectRatio,
                    delays: emoteInfo.frame_delays,
                    tex: this.atlasTex
                        ? {
                            map: this.map,
                            x_size: this.atlasTex.x_size,
                            y_size: this.atlasTex.y_size,
                            delays: emoteInfo.frame_delays
                        }
                        : this.map
                });
                if (onLoad) {
                    onLoad(this);
                }
            });
        });
    }
    animateTexture(timestamp) {
        if (this.atlasTex !== undefined) {
            return this.atlasTex.animate(timestamp);
        }
        return undefined;
    }
}
//# sourceMappingURL=material.js.map