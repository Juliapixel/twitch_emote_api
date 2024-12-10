import { Loader } from "three";
import { EmoteObject } from "./emote";
export class EmoteLoader extends Loader {
    constructor(manager, apiUrl, materialKind) {
        super(manager);
        this.path = apiUrl;
        this.materialKind = materialKind;
    }
    load(emote, onLoad, onProgress, onError) {
        try {
            new EmoteObject(emote.source, this.path, emote, this.materialKind, (e) => onLoad(e));
        }
        catch (e) {
            if (onError) {
                onError(e);
            }
        }
    }
    loadAsync(emote, onProgress) {
        return new Promise((resolve, reject) => {
            this.load(emote, (obj) => resolve(obj), onProgress, (err) => reject(err));
        });
    }
}
//# sourceMappingURL=loader.js.map