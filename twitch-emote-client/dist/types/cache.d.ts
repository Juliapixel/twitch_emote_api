export declare class Cache<K, V> {
    map: Map<K, V | Promise<V>>;
    constructor();
    add(key: K, value: V | Promise<V>): void;
    get(key: K): V | Promise<V> | undefined;
}
//# sourceMappingURL=cache.d.ts.map