import { Loader } from "three";
import { EmoteObject } from "./emote";
export class EmoteLoader extends Loader {
    constructor(manager, apiUrl) {
        super(manager);
        this.path = apiUrl;
    }
    load(emote, onLoad, onProgress, onError) {
        try {
            new EmoteObject(emote.source, this.path, emote, (e) => onLoad(e));
        }
        catch (e) {
            if (onError) {
                onError(e);
            }
        }
    }
}
//# sourceMappingURL=loader.js.map