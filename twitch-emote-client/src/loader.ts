import { Loader } from "three";
import { EmoteObject } from "./emote";
import { LoadingManager } from "three";
import { CallbackEmoteInfo, ChannelEmote } from "./client";

export class EmoteLoader extends Loader<EmoteObject, CallbackEmoteInfo> {
    constructor(manager: LoadingManager | undefined, apiUrl: string) {
        super(manager);
        this.path = apiUrl;
    }

    load(
        emote: CallbackEmoteInfo,
        onLoad: (data: EmoteObject) => void,
        onProgress?: (event: ProgressEvent) => void,
        onError?: (err: unknown) => void
    ): void {
        try {
            new EmoteObject(emote.source, this.path, emote, (e) => onLoad(e));
        } catch (e) {
            if (onError) {
                onError(e);
            }
        }
    }
}
