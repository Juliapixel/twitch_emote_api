import { Loader } from "three";
import { EmoteObject } from "./emote";
import { LoadingManager } from "three";
import { CallbackEmoteInfo } from "./client";
import { MaterialKind } from "./material";
import { EmoteTexture } from "./texture";
/**
 * class for loading a Twitch, 7TV, BTTV or FFZ emote, uses
 * [TextureLoader](https://threejs.org/docs/index.html#api/en/loaders/TextureLoader)
 * and {@link EmoteTextureLoader} internally
 */
export declare class EmoteLoader extends Loader<EmoteObject, CallbackEmoteInfo> {
    materialKind: MaterialKind;
    textureLoader: EmoteTextureLoader;
    constructor(apiUrl: string, materialKind: MaterialKind, manager?: LoadingManager);
    load(emoteInfo: CallbackEmoteInfo, onLoad: (data: EmoteObject) => void, onProgress?: (event: ProgressEvent) => void, onError?: (err: unknown) => void): void;
    loadAsync(emoteInfo: CallbackEmoteInfo, onProgress?: (event: ProgressEvent) => void): Promise<EmoteObject>;
}
export declare class EmoteTextureLoader extends Loader<EmoteTexture, CallbackEmoteInfo> {
    apiUrl: string;
    private innerTextureLoader;
    constructor(apiUrl: string, manager?: LoadingManager);
    load(emoteInfo: CallbackEmoteInfo, onLoad: (data: EmoteTexture) => void, onProgress?: (event: ProgressEvent) => void, onError?: (err: unknown) => void): void;
    loadAsync(emoteInfo: CallbackEmoteInfo, onProgress?: (event: ProgressEvent) => void): Promise<EmoteTexture>;
}
//# sourceMappingURL=loader.d.ts.map