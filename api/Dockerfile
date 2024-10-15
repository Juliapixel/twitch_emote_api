FROM rustlang/rust:nightly AS build
WORKDIR /app
ADD . .
RUN cargo build -r

FROM debian:bookworm
COPY --from=build /app/target/release/twitch_emote_api /usr/bin/
CMD [ "twitch_emote_api" ]
