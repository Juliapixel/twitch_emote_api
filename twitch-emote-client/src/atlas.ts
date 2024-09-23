import { Vector2 } from "three";

export class AtlasTexture {
    public x_size: number;
    public y_size: number;
    private delays: number[];
    private animationLength: number

    constructor(x_size: number, y_size: number, delays: number[]) {
        this.x_size = x_size;
        this.y_size = y_size;
        this.delays = delays;
        this.animationLength = delays.reduce((sum, cur) => sum += cur)
    }

    animate(timestamp: number) {
        let currentDelay = timestamp % this.animationLength;
        let delaySum = 0;

        let i = 0;
        for (const delay of this.delays) {
            delaySum += delay;
            if (delaySum > currentDelay) {
                const x_pos = Math.floor(i % this.x_size);
                const y_pos = Math.floor(i / this.x_size);
                const x_step = 1.0 / this.x_size;
                const y_step = 1.0 / this.y_size;
                const tl_corner = [x_pos * x_step, y_pos * y_step];
                const br_corner = [(x_pos * x_step) + x_step, (y_pos * y_step) + y_step];

                tl_corner[1] = (tl_corner[1] - 1) * -1
                br_corner[1] = (br_corner[1] - 1) * -1

                return [
                    new Vector2(tl_corner[0], tl_corner[1]), // TL corner
                    new Vector2(br_corner[0], tl_corner[1]), // TR corner
                    new Vector2(tl_corner[0], br_corner[1]), // BL corner
                    new Vector2(br_corner[0], br_corner[1]), // BR corner
                ]
            }
            i++;
        }
        throw new Error("frame not found? wtf?")
    }
}
