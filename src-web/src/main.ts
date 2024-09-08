import TwitchChat from "twitch-chat-emotes-threejs";
import Chat from "twitch-chat-emotes";
import Stats from 'three/examples/jsm/libs/stats.module.js'
import "./style.css";

import { Client } from "tmi.js";
import { Sprite } from "three";
import { Vector3 } from "three";
import { Group } from "three";
import { SpriteMaterial } from "three";
import { PerspectiveCamera } from "three";
import { Scene } from "three";
import { WebGLRenderer } from "three";


/**
 * URL Parameters and configuration
 */
// a default array of twitch channels to join
let channels = ['julialuxel'];

// the following few lines of code will allow you to add ?channels=channel1,channel2,channel3 to the URL in order to override the default array of channels
const params = new URL(window.location.toString()).searchParams;

if (params.has('channels') || params.has('channel')) {
	const temp = params.get('channels') + ',' + params.get('channel');
	channels = temp.split(',').filter(value => value.length > 0 && value !== 'null');
}

// performance stats enabled using ?stats=true in the browser URL
let stats;
if (params.get('stats') === 'true') {
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
const renderer = new WebGLRenderer({ antialias: false });
renderer.setSize(window.innerWidth, window.innerHeight);



// separate from three.js hierarchy, we want to keep track of emotes
// to update them with custom logic every render tick
declare module 'three' {
	interface Group {
		update? (): void;
		data: {
			timestamp: number;
			lifetime?: number;
			lifespan: number;
			velocity: Vector3;
		}
	}
}
const sceneEmoteArray: Group[] = [];

function resize() {
	camera.aspect = window.innerWidth / window.innerHeight;
	camera.updateProjectionMatrix();
	renderer.setSize(window.innerWidth, window.innerHeight);
}

window.addEventListener('DOMContentLoaded', () => {
	window.addEventListener('resize', resize);
	if (stats) document.body.appendChild(stats.dom);
	document.body.appendChild(renderer.domElement);
	draw();
})



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
			element.updateMatrix();
		}
	}

	renderer.render(scene, camera);
	if (stats) stats.end();
};



/*
** Twitch chat configuration
*/

const chat = new Client({channels: channels})
chat.connect()
chat.on("message", (channel, state, msg, self) => {
	console.log(`${state["display-name"]}: ${msg}`)
})

/*
** Handle Twitch Chat Emotes
*/
const spawnEmote = (emotes) => {
	//prevent lag caused by emote buildup when you tab out from the page for a while
	if (performance.now() - lastFrame > 1000) return;

	const group = new Group();
	group.data = {
		lifespan: 5000,
		timestamp: Date.now(),
		velocity: new Vector3(
			(Math.random() - 0.5) * 2,
			(Math.random() - 0.5) * 2,
			(Math.random() - 0.5) * 2
		).normalize(),
	}

	let i = 0;
	emotes.forEach((emote) => {
		const sprite = new Sprite(emote.material);
		// ensure emotes from the same message don't overlap each other
		sprite.position.x = i;

		group.add(sprite);
		i++;
	})

	group.update = () => { // called every frame
		let progress = (Date.now() - group.data.timestamp) / group.data.lifespan;
		if (progress < 0.25) { // grow to full size in first quarter
			group.scale.setScalar(progress * 4);
		} else if (progress > 0.75) { // shrink to nothing in last quarter
			group.scale.setScalar((1 - progress) * 4);
		} else { // maintain full size in middle
			group.scale.setScalar(1);
		}
	}

	scene.add(group);
	sceneEmoteArray.push(group);
};

// ChatInstance.listen(spawnEmote);

// spawn some fake emotes for testing purposes
const placeholder_mats = [
	new SpriteMaterial({ color: 0xff4444 }),
	new SpriteMaterial({ color: 0x44ff44 }),
	new SpriteMaterial({ color: 0x4444ff }),
]
setInterval(() => {
	spawnEmote([{
		material: placeholder_mats[Math.floor(Math.random() * placeholder_mats.length)]
	}]);
}, 1000);
