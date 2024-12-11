import { LinearMipMapNearestFilter, Loader, NearestFilter, SRGBColorSpace, Texture, TextureLoader } from "three";
import { EmoteObject } from "./emote";
import { EmoteBasicMaterial, EmoteStandardMaterial, MaterialKind } from "./material";
import { AtlasTextureInfo } from "./texture";
import { Cache } from "./cache";
/**
 * class for loading a Twitch, 7TV, BTTV or FFZ emote, uses
 * [TextureLoader](https://threejs.org/docs/index.html#api/en/loaders/TextureLoader)
 * and {@link EmoteTextureLoader} internally
 */
export class EmoteLoader extends Loader {
    constructor(apiUrl, materialKind, manager) {
        super(manager);
        this.path = apiUrl;
        this.textureLoader = new EmoteTextureLoader(apiUrl, manager);
        if (materialKind === undefined) {
            this.materialKind = MaterialKind.Basic;
        }
        this.materialKind = materialKind;
    }
    load(emoteInfo, onLoad, onProgress, onError) {
        this.textureLoader.load(emoteInfo, (tex) => {
            let material;
            switch (this.materialKind) {
                case MaterialKind.Basic:
                    material = new EmoteBasicMaterial({ map: tex });
                    break;
                case MaterialKind.Standard:
                    material = new EmoteStandardMaterial({ map: tex });
                    break;
            }
            onLoad(new EmoteObject(material));
        }, onProgress, onError);
    }
    loadAsync(emoteInfo, onProgress) {
        return new Promise((resolve, reject) => {
            this.load(emoteInfo, (obj) => resolve(obj), onProgress, (err) => reject(err));
        });
    }
}
const textureCache = new Cache();
function emoteInfoToKey(emoteInfo) {
    return emoteInfo.platform + "," + emoteInfo.id;
}
export class EmoteTextureLoader extends Loader {
    constructor(apiUrl, manager) {
        super(manager);
        this.apiUrl = apiUrl;
        this.innerTextureLoader = new TextureLoader(this.manager);
    }
    load(emoteInfo, onLoad, onProgress, onError) {
        let hit = textureCache.get(emoteInfoToKey(emoteInfo));
        if (hit instanceof Promise) {
            hit.then((tex) => onLoad(tex));
            if (onError) {
                hit.catch((e) => onError(e));
            }
            return;
        }
        else if (hit instanceof Texture) {
            onLoad(hit);
        }
        textureCache.add(emoteInfoToKey(emoteInfo), new Promise((resolve, reject) => {
            let urlPrefix;
            switch (emoteInfo.source) {
                case "globals":
                case "global":
                    urlPrefix = `${this.apiUrl}/emote/globals/${emoteInfo.platform}`;
                    break;
                case "twitch_emote":
                    urlPrefix = `${this.apiUrl}/emote/twitch`;
                    break;
                default:
                    urlPrefix = `${this.apiUrl}/emote/${emoteInfo.source.replace(/^\#/, "")}`;
            }
            let texUrl = emoteInfo.animated
                ? `${urlPrefix}/${emoteInfo.name}/atlas.webp`
                : `${urlPrefix}/${emoteInfo.name}/0.webp`;
            fetch(`${urlPrefix}/${emoteInfo.name}`)
                .then(async (resp) => {
                let einfo = await resp.json();
                this.innerTextureLoader
                    .loadAsync(texUrl)
                    .then((tex) => {
                    if (einfo.atlas_info !== undefined) {
                        tex.atlasInfo =
                            new AtlasTextureInfo(einfo.atlas_info.x_size, einfo.atlas_info.y_size, einfo.frame_delays);
                    }
                    tex.aspectRatio =
                        einfo.width / einfo.height;
                    tex.magFilter = NearestFilter;
                    tex.minFilter = LinearMipMapNearestFilter;
                    tex.colorSpace = SRGBColorSpace;
                    const eTex = tex;
                    resolve(eTex);
                    if (onLoad) {
                        onLoad(eTex);
                    }
                })
                    .catch((e) => reject(e));
            })
                .catch((e) => reject(e));
        }));
    }
    loadAsync(emoteInfo, onProgress) {
        return new Promise((resolve, reject) => {
            this.load(emoteInfo, (t) => resolve(t), onProgress, (e) => reject(e));
        });
    }
}
//# sourceMappingURL=loader.js.map