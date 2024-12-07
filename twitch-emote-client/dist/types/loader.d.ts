import { Loader } from "three";
import { EmoteObject } from "./emote";
import { LoadingManager } from "three";
import { CallbackEmoteInfo } from "./client";
export declare class EmoteLoader extends Loader<EmoteObject, CallbackEmoteInfo> {
    constructor(manager: LoadingManager | undefined, apiUrl: string);
    load(emote: CallbackEmoteInfo, onLoad: (data: EmoteObject) => void, onProgress?: (event: ProgressEvent) => void, onError?: (err: unknown) => void): void;
}
//# sourceMappingURL=loader.d.ts.map