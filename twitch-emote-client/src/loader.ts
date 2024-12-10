import { Loader } from "three";
import { EmoteObject } from "./emote";
import { LoadingManager } from "three";
import { CallbackEmoteInfo } from "./client";
import { MaterialKind } from "./material";

export class EmoteLoader extends Loader<EmoteObject, CallbackEmoteInfo> {
    materialKind: MaterialKind;

    constructor(
        manager: LoadingManager | undefined,
        apiUrl: string,
        materialKind: MaterialKind
    ) {
        super(manager);
        this.path = apiUrl;
        this.materialKind = materialKind;
    }

    load(
        emote: CallbackEmoteInfo,
        onLoad: (data: EmoteObject) => void,
        onProgress?: (event: ProgressEvent) => void,
        onError?: (err: unknown) => void
    ): void {
        try {
            new EmoteObject(emote.source, this.path, emote, this.materialKind, (e) =>
                onLoad(e)
            );
        } catch (e) {
            if (onError) {
                onError(e);
            }
        }
    }

    loadAsync(
        emote: CallbackEmoteInfo,
        onProgress?: (event: ProgressEvent) => void
    ): Promise<EmoteObject> {
        return new Promise((resolve, reject) => {
            this.load(
                emote,
                (obj) => resolve(obj),
                onProgress,
                (err) => reject(err)
            );
        });
    }
}
