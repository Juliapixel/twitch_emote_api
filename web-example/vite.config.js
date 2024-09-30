import { resolveConfig } from "vite";
import { resolve } from "path";

/** @type {import("vite").UserConfig;} */
export default {
    resolve: {
        conditions: ["import"]
    },
    build: {
        rollupOptions: {
            input: {
                main: resolve(__dirname, "index.html"),
                desert: resolve(__dirname, "desert/index.html")
            }
        }
    },
    appType: "mpa"
};
