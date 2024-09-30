import Stats from "three/examples/jsm/libs/stats.module.js";
import "./style.css";

import {
    AgXToneMapping,
    AmbientLight,
    AnimationClip,
    AnimationMixer,
    Color,
    DirectionalLight,
    Group,
    Light,
    LoadingManager,
    MathUtils,
    Mesh,
    PCFShadowMap,
    PerspectiveCamera,
    Scene,
    Vector2,
    Vector3,
    WebGLRenderer
} from "three";
import { EmotesClient, EmoteObject, CallbackEmoteInfo } from "twitch-emote-client";
import {
    EffectComposer,
    GLTFLoader,
    OutputPass,
    RenderPass,
    Sky,
    SMAAPass,
    SSAOPass,
    UnrealBloomPass
} from "three/examples/jsm/Addons.js";

// a default array of twitch channels to join
let channels: string[] = [];

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
let emoteCountPanel: Stats.Panel | undefined;
if (params.get("stats") === "true") {
    stats = new Stats();
    emoteCountPanel = new Stats.Panel("EMOTES", "#f5b942", "#523909");
    stats.addPanel(emoteCountPanel);
    stats.showPanel(0);
    document.body.appendChild(stats.dom);
}

/*
 ** Initiate ThreeJS scene
 */

let camera = new PerspectiveCamera(
    20,
    window.innerWidth / window.innerHeight,
    0.1,
    1000
);
camera.position.z = 5;

const scene = new Scene();

let loadingManager = new LoadingManager();
let gltfLoader = new GLTFLoader(loadingManager);

let sunDir = new Vector3();
let walkAnim: AnimationClip;

// load the scene from blender
gltfLoader.load("/cacti.glb", (glb) => {
    // things this animation are applied to must all be called "root" or it no
    // workie
    walkAnim = glb.animations[0];
    walkAnim.tracks.forEach((val) => {
        val.name = val.name.replace("Plane001", "root");
    });

    camera.position.copy(glb.cameras[0].position);
    camera.rotation.copy(glb.cameras[0].rotation);

    glb.scene.castShadow = true;
    glb.scene.receiveShadow = true;

    glb.scene.traverse((obj) => {
        // blender lights are really fucking strong
        if (obj instanceof Light) {
            obj.intensity *= 0.001;
        }

        if (obj instanceof Mesh) {
            obj.castShadow = true;
            obj.receiveShadow = true;
        }

        // the sun......
        if (obj instanceof DirectionalLight) {
            obj.getWorldDirection(sunDir);

            obj.castShadow = true;
            obj.receiveShadow = false;

            obj.shadow.radius = 6;
            obj.shadow.mapSize = new Vector2(2048, 2048);
            // need this to make it cover the whole scene
            obj.shadow.camera.scale.multiplyScalar(3.5);
            obj.shadow.camera.near = 0.1;
            obj.shadow.bias = -0.0005;
            obj.shadow.camera.far = 50;
        }
    });

    scene.add(glb.scene);
});

scene.add(new AmbientLight("#2445FF", 10));
scene.background = new Color("#6899d9");

let sky = new Sky();
sky.scale.setScalar(1000);
const sunPosition = sunDir;

sky.material.uniforms.sunPosition.value = sunPosition;

// scene.add(sky);

const renderer = new WebGLRenderer({ antialias: false });

renderer.setSize(window.innerWidth, window.innerHeight);
// same as blender
renderer.toneMapping = AgXToneMapping;

renderer.shadowMap.enabled = true;
renderer.shadowMap.type = PCFShadowMap;

// cool post-processing
let composer = new EffectComposer(renderer);
composer.setSize(window.innerWidth, window.innerHeight);

composer.addPass(new RenderPass(scene, camera));

let ssaoPass = new SSAOPass(scene, camera, window.innerWidth, window.innerHeight);
composer.addPass(ssaoPass);

let smaaPass = new SMAAPass(window.innerWidth, window.innerHeight);
composer.addPass(smaaPass);

let bloomPass = new UnrealBloomPass(
    new Vector2(window.innerWidth, window.innerHeight),
    0.2,
    0.4,
    5
);
composer.addPass(bloomPass);

composer.addPass(new OutputPass());

// separate from three.js hierarchy, we want to keep track of emotes
// to update them with custom logic every render tick
declare module "three" {
    interface Group {
        updateAnim: (deltaTime: number) => void;
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
    let width = window.innerWidth;
    let height = window.innerHeight;

    renderer.setSize(width, height);
    composer.setSize(width, height);
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
        if (element.data.timestamp + element.data.lifespan < Date.now()) {
            sceneEmoteArray.splice(index, 1);
            scene.remove(element);
        } else if (element.updateAnim) {
            element.updateAnim(delta);
        }
    }

    composer.render(delta);

    // update stats and shit
    if (stats && emoteCountPanel) {
        stats.end();
        if (sceneEmoteArray.length > 0) {
            emoteCountPanel.update(
                sceneEmoteArray
                    .map((group) => group.children.length)
                    .reduce((sum, cur) => (sum += cur)),
                50
            );
        } else {
            emoteCountPanel.update(0, 50);
        }
    }
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
    group.data = {
        lifespan: 0,
        timestamp: Date.now(),
        velocity: new Vector3(Math.random() - 0.5, Math.random() - 0.5, 0)
            .normalize()
            .multiply(new Vector3(2, 2, 1))
    };

    let slicedEmotes = emotes.slice(0, 12);
    let processedEmotes = 0;
    let i = 0;
    for (const emote of slicedEmotes) {
        // gotta do this cuz new EmoteObject takes the wrong i for some reason!
        let curI = i;
        i++;
        new EmoteObject(emote.source, client.config.emotesApi, emote, (obj) => {
            let ratio = 0;
            if (slicedEmotes.length !== 1) {
                ratio = curI / slicedEmotes.length - 1;
            }

            obj.castShadow = true;
            obj.receiveShadow = true;
            obj.material.toneMapped = false;

            // conga line
            obj.position.y = -0.5 * curI;
            // make it not float or sink
            obj.position.z = 0.6;
            // make it point the right way
            obj.rotateX(-90);

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

    // so the animations work
    group.name = "root";

    let mixer = new AnimationMixer(group);
    let action = mixer.clipAction(walkAnim);
    action.play();

    group.data.lifespan = walkAnim.duration * 1000;

    group.updateAnim = (deltaTime: number) => {
        for (let child of group.children) {
            if (child instanceof EmoteObject) {
                child.animateTexture(
                    (performance.now() + group.data.timestamp) / 1000
                );
            }
        }
        mixer.update(deltaTime);
    };
};
