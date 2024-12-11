export class Cache {
    constructor() {
        this.map = new Map();
    }
    add(key, value) {
        this.map.set(key, value);
    }
    get(key) {
        return this.map.get(key);
    }
}
//# sourceMappingURL=cache.js.map