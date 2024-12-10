import { Loader } from "three";
import { EmoteObject } from "./emote";
import { LoadingManager } from "three";
import { CallbackEmoteInfo } from "./client";
import { MaterialKind } from "./material";
export declare class EmoteLoader extends Loader<EmoteObject, CallbackEmoteInfo> {
    materialKind: MaterialKind;
    constructor(manager: LoadingManager | undefined, apiUrl: string, materialKind: MaterialKind);
    load(emote: CallbackEmoteInfo, onLoad: (data: EmoteObject) => void, onProgress?: (event: ProgressEvent) => void, onError?: (err: unknown) => void): void;
    loadAsync(emote: CallbackEmoteInfo, onProgress?: (event: ProgressEvent) => void): Promise<EmoteObject>;
}
//# sourceMappingURL=loader.d.ts.map