# twitch-emote-client

it's a library that helps you use twitch chat emotes (native, BTTV, FFZ and 7TV)
in your three.js projects POGGERS

## Usage

basic stuff like:
```js
let client = new EmotesClient({ channels: ["julialuxel"] });
let loader = new EmoteLoader(new LoadingManager(), client.config.emotesApi);

client.on("emote", (emotes, source) => {
    for (const emote of emotes) {
        let emote = await loader.loadAsync(emote);

        // do stuff here with ur awesome new EmoteObject like:
        emote.scale.multiplyScalar(1000000000);
    }
});
```

also don't forget to call `EmoteObject.animateTexture(timestamp)` to make animated
emotes move

also some basic usage examples [here](https://github.com/Juliapixel/twitch_emote_api/blob/main/web-example)
