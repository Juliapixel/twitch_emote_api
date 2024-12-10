import {
    LinearMipMapNearestFilter,
    LoadingManager,
    MeshBasicMaterial,
    MeshStandardMaterial,
    NearestFilter,
    SRGBColorSpace,
    Texture,
    TextureLoader,
    Vector2
} from "three";
import { ChannelEmote } from "./client.js";
import { AtlasTexture } from "./atlas.js";

/** kind of material used for the emote object */
export enum MaterialKind {
    /** MeshBasicMaterial */
    Basic,
    /** MeshStandardMaterial */
    Standard
}

interface Atlas {
    map: Texture;
    x_size: number;
    y_size: number;
    delays: number[];
}

let cache: Map<
    string,
    {
        tex: Atlas | Texture;
        aspectRatio: number;
        animationLength: number;
        delays: number[];
    }
> = new Map();

const loadingManager = new LoadingManager();

interface IEmoteMaterial {
    animationLength: number;
    aspectRatio: number;
    isAnimated: boolean;
    atlasTex?: AtlasTexture;

    animateTexture(timestamp: number): Vector2[] | undefined;
}

export class EmoteBasicMaterial extends MeshBasicMaterial implements IEmoteMaterial {
    public animationLength: number = 0;
    public aspectRatio: number = 1;
    public isAnimated: boolean = false;
    public atlasTex?: AtlasTexture;

    constructor(
        source: string,
        emote: ChannelEmote,
        apiUrl: string,
        onLoad?: (mat: EmoteBasicMaterial) => void | Promise<void>
    ) {
        super({ transparent: true });

        let hit = cache.get(`channel:${source},emote:${emote.name}`);
        if (hit) {
            Object.assign(this, hit);
            if (hit.tex instanceof Texture) {
                this.map = hit.tex;
                this.isAnimated = false;
            } else {
                this.map = hit.tex.map;
                this.atlasTex = new AtlasTexture(
                    hit.tex.x_size,
                    hit.tex.y_size,
                    hit.tex.delays
                );
                this.isAnimated = true;
            }
            if (onLoad) {
                onLoad(this);
            }
            return;
        }

        let urlPrefix: string;
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
            let emoteInfo: ChannelEmote = await resp.json();
            this.isAnimated = emoteInfo.frame_count > 1;
            this.animationLength = emoteInfo.frame_delays.reduce(
                (sum, delay) => (sum += delay)
            );

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
                    this.atlasTex = new AtlasTexture(
                        emoteInfo.atlas_info.x_size,
                        emoteInfo.atlas_info.y_size,
                        emoteInfo.frame_delays
                    );
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

    animateTexture(timestamp: number): Vector2[] | undefined {
        if (this.atlasTex !== undefined) {
            return this.atlasTex.animate(timestamp);
        }
        return undefined;
    }
}

export class EmoteStandardMaterial
    extends MeshStandardMaterial
    implements IEmoteMaterial
{
    public animationLength: number = 0;
    public aspectRatio: number = 1;
    public isAnimated: boolean = false;
    public atlasTex?: AtlasTexture;

    constructor(
        source: string,
        emote: ChannelEmote,
        apiUrl: string,
        onLoad?: (mat: EmoteStandardMaterial) => void | Promise<void>
    ) {
        super({ transparent: true });

        let hit = cache.get(`channel:${source},emote:${emote.name}`);
        if (hit) {
            Object.assign(this, hit);
            if (hit.tex instanceof Texture) {
                this.map = hit.tex;
                this.isAnimated = false;
            } else {
                this.map = hit.tex.map;
                this.atlasTex = new AtlasTexture(
                    hit.tex.x_size,
                    hit.tex.y_size,
                    hit.tex.delays
                );
                this.isAnimated = true;
            }
            if (onLoad) {
                onLoad(this);
            }
            return;
        }

        let urlPrefix: string;
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
            let emoteInfo: ChannelEmote = await resp.json();
            this.isAnimated = emoteInfo.frame_count > 1;
            this.animationLength = emoteInfo.frame_delays.reduce(
                (sum, delay) => (sum += delay)
            );

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
                    this.atlasTex = new AtlasTexture(
                        emoteInfo.atlas_info.x_size,
                        emoteInfo.atlas_info.y_size,
                        emoteInfo.frame_delays
                    );
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

    animateTexture(timestamp: number): Vector2[] | undefined {
        if (this.atlasTex !== undefined) {
            return this.atlasTex.animate(timestamp);
        }
        return undefined;
    }
}
