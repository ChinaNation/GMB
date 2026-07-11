FROM docker.io/paritytech/ci-unified:latest as builder

WORKDIR /citizenchain
COPY . /citizenchain

RUN cargo fetch
RUN cargo build --locked --release

FROM docker.io/parity/base-bin:latest

COPY --from=builder /citizenchain/target/release/citizenchain /usr/local/bin

USER root
RUN useradd -m -u 1001 -U -s /bin/sh -d /citizenchain citizenchain && \
	mkdir -p /data /citizenchain/.local/share && \
	chown -R citizenchain:citizenchain /data && \
	ln -s /data /citizenchain/.local/share/citizenchain && \
# unclutter and minimize the attack surface
	rm -rf /usr/bin /usr/sbin && \
# check if executable works in this container
	/usr/local/bin/citizenchain --version

USER citizenchain

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/citizenchain"]
