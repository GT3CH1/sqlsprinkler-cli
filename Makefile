check:
	@cargo check

build:
	@cargo build

install: build-rpi
	cp target/armv7-unknown-linux-gnueabihf/debug/sqlsprinkler-cli /usr/bin/sqlsprinkler

build-rpi:
	@cross build --target armv7-unknown-linux-gnueabihf
