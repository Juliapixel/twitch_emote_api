import { resolveConfig } from "vite";

/** @type {import("vite").UserConfig;} */
export default {
    resolve: {
        conditions: ["import"]
    }
}
