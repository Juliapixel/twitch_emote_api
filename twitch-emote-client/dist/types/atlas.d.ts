import { Vector2 } from "three";
export declare class AtlasTexture {
    x_size: number;
    y_size: number;
    private delays;
    private animationLength;
    constructor(x_size: number, y_size: number, delays: number[]);
    animate(timestamp: number): Vector2[];
}
//# sourceMappingURL=atlas.d.ts.map