import Stats from "three/examples/jsm/libs/stats.module.js";
import "./style.css";

import {
    Group,
    Mesh,
    PerspectiveCamera,
    PlaneGeometry,
    Scene,
    SpriteMaterial,
    Vector3,
    WebGLRenderer
} from "three";
import { ChannelEmote, EmotesClient, EmoteMaterial, EmoteObject, CallbackEmoteInfo } from "twitch-emote-client";

// a default array of twitch channels to join
let channels = ["julialuxel"];

// the following few lines of code will allow you to add ?channels=channel1,channel2,channel3 to the URL in order to override the default array of channels
const params = new URL(window.location.toString()).searchParams;

if (params.has("channels") || params.has("channel")) {
    const temp = params.get("channels") + "," + params.get("channel");
    channels = temp
        .split(",")
        .filter((value) => value.length > 0 && value !== "null");
}

// performance stats enabled using ?stats=true in the browser URL
let stats: Stats | undefined;
if (params.get("stats") === "true") {
    stats = new Stats();
    stats.showPanel(1);
    document.body.appendChild(stats.dom);
}

/*
 ** Initiate ThreeJS scene
 */

const camera = new PerspectiveCamera(
    70,
    window.innerWidth / window.innerHeight,
    0.1,
    1000
);
camera.position.z = 5;

const scene = new Scene();
const renderer = new WebGLRenderer({ antialias: true, alpha: true });
renderer.setSize(window.innerWidth, window.innerHeight);

// separate from three.js hierarchy, we want to keep track of emotes
// to update them with custom logic every render tick
declare module "three" {
    interface Group {
        update?(): void;
        data: {
            timestamp: number;
            lifetime?: number;
            lifespan: number;
            velocity: Vector3;
        };
    }
}
const sceneEmoteArray: Group[] = [];

function resize() {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
}

window.addEventListener("DOMContentLoaded", () => {
    window.addEventListener("resize", resize);
    if (stats) document.body.appendChild(stats.dom);
    document.body.appendChild(renderer.domElement);
    draw();
});

/*
 ** Draw loop
 */

let lastFrame = performance.now();
function draw() {
    if (stats) stats.begin();
    requestAnimationFrame(draw);
    const delta = Math.min(1, Math.max(0, (performance.now() - lastFrame) / 1000));
    lastFrame = performance.now();

    for (let index = sceneEmoteArray.length - 1; index >= 0; index--) {
        const element = sceneEmoteArray[index];
        element.position.addScaledVector(element.data.velocity, delta);
        if (element.data.timestamp + element.data.lifespan < Date.now()) {
            sceneEmoteArray.splice(index, 1);
            scene.remove(element);
        } else {
            if (element.update) {
                element.update();
            }
            element.updateMatrix();
        }
    }

    renderer.render(scene, camera);
    if (stats) stats.end();
}

/*
 ** Twitch chat configuration
 */

let client = new EmotesClient({ channels: channels });
client.on("emote", (emotes, channel) => {
    spawnEmote(emotes, channel);
});

/*
 ** Handle Twitch Chat Emotes
 */
const spawnEmote = (emotes: CallbackEmoteInfo[], channel: string) => {
    //prevent lag caused by emote buildup when you tab out from the page for a while
    if (performance.now() - lastFrame > 1000) return;

    const group = new Group();
    group.position.setZ(-3);
    group.data = {
        lifespan: 5000,
        timestamp: Date.now(),
        velocity: new Vector3(
            Math.random() - 0.5,
            Math.random() - 0.5,
            0
        )
            .normalize()
            .multiply(new Vector3(2, 2, 1))
    };

    let slicedEmotes = emotes.slice(0, 12);
    let processedEmotes = 0;
    let i = 0;
    for (const emote of slicedEmotes) {
        // gotta do this cuz new EmoteObject takes the wrong i for some reason!
        let curI = i;
        i++
        new EmoteObject(emote.source, client.config.emotesApi, emote, (obj) => {
            let ratio = 0;
            if (slicedEmotes.length !== 1) {
                ratio = curI / slicedEmotes.length - 1;
            }

            obj.position.x = (Math.random() * 4 - 2) * ratio ;
            obj.position.y = (Math.random() * 4 - 2) * ratio ;
            obj.position.z = (Math.random() * 4 - 2) * ratio ;

            group.add(obj);
            processedEmotes++;
            // wait until all emotes have been processed properly to spawn the group
            // also SURELY this doesnt cause any race conditions right
            if (processedEmotes === slicedEmotes.length) {
                group.data.timestamp = Date.now();
                scene.add(group);
                sceneEmoteArray.push(group);
            }
        });

    }

    group.update = () => {
        for (let child of group.children) {
            if (child instanceof EmoteObject) {
                child.animateTexture(
                    (performance.now() + group.data.timestamp) / 1000
                );
            }
        }
        let progress = (Date.now() - group.data.timestamp) / group.data.lifespan;
        if (progress < 0.25) {
            // grow to full size in first quarter
            group.scale.setScalar(progress * 4);
        } else if (progress > 0.75) {
            // shrink to nothing in last quarter
            group.scale.setScalar((1 - progress) * 4);
        } else {
            // maintain full size in middle
            group.scale.setScalar(1);
        }
    };
};

setInterval(() => {
    spawnEmote(
        [{ id: "64cd931ed3cf2f1c8cca5264", name: "juh", platform: "7tv", source: "julialuxel" }],
        "julialuxel"
    );
}, 1000);
