# This is the build stage for Airo. Here we create the binary in a temporary image.
FROM docker.io/paritytech/ci-linux:production AS builder

WORKDIR /airo
COPY . /airo
RUN RUSTUP_PERMIT_COPY_RENAME=1 cargo build --locked --profile=production

# This is the 2nd stage: a very small image where we copy the Airo binary."
FROM docker.io/library/ubuntu:24.04

COPY --from=builder /airo/target/production/airo /usr/local/bin

RUN useradd -m -u 1001 -U -s /bin/sh -d /airo airo && \
	mkdir -p /data /airo/.local/share && \
	chown -R airo:airo /data && \
	ln -s /data /airo/.local/share/airo && \
# unclutter and minimize the attack surface
	rm -rf /usr/bin /usr/sbin && \
# check if executable works in this container
	/usr/local/bin/airo --version

USER airo

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/airo"]
