export class Cache<K, V> {
    map: Map<K, V | Promise<V>>;

    constructor() {
        this.map = new Map();
    }

    add(key: K, value: V | Promise<V>) {
        this.map.set(key, value);
    }

    get(key: K): V | Promise<V> | undefined {
        return this.map.get(key);
    }
}
