UNAME := $(shell uname -m)
VERSION := 0.1-6
ROOT := sqlsprinkler_$(VERSION)_$(ARCH)
default:
	$(MAKE) build

check:
	@echo "Checking for armv7l on" $(UNAME)
    ifeq ($(UNAME), x86_64)
	    @cross clippy --target armv7-unknown-linux-gnueabihf
    else
		cargo clippy
    endif

test:
	@echo "Checking for armv7l on" $(UNAME)
    ifeq ($(UNAME), x86_64)
	    @cross test --target armv7-unknown-linux-gnueabihf
    else
		cargo test
    endif

fmt:
	@echo "Formatting"
	cargo fmt

build:
	@echo "Building for arm on" $(UNAME)
    ifeq ($(UNAME), x86_64)
		$(MAKE) build-arm
    else
		@cargo build --release
    endif

build-arm:
	@cross build --target arm-unknown-linux-gnueabihf --release

install-service:
	cp -v systemd/sqlsprinkler-daemon.service /etc/systemd/system

install: deb
	dpkg -i $(ROOT).deb

clean:
	-rm *.deb
	-rm -rf ./$(ROOT)/
	cargo clean

deb: build-arm
	@install -dm755 $(ROOT)
	@install -Dm755 target/arm-unknown-linux-gnueabihf/release/sqlsprinkler-cli $(ROOT)/usr/bin/sqlsprinkler
	@install -Dm755 conf/sqlsprinkler.conf $(ROOT)/etc/sqlsprinkler/sqlsprinkler.conf
	@install -Dm755 systemd/sqlsprinkler-daemon.service $(ROOT)/lib/systemd/system/sqlsprinkler-daemon.service
	@install -Dm755 systemd/sqlsprinkler-mqtt-daemon.service $(ROOT)/lib/systemd/system/sqlsprinkler-mqtt-daemon.service
	@install -dm755 $(ROOT)/DEBIAN
	@touch $(ROOT)/DEBIAN/control
	@install -Dm755 conf/preinst $(ROOT)/DEBIAN/preinst
	@install -Dm755 conf/postinst $(ROOT)/DEBIAN/postinst
	@install -Dm755 conf/conffiles $(ROOT)/DEBIAN/conffiles
	@echo "Package: sqlsprinkler\n\
Version: $(VERSION)\n\
Architecture: armhf\n\
Maintainer: Gavin Pease <gavinpease@gmail.com>\n\
Description: The command line and daemon for sqlsprinkler" > $(ROOT)/DEBIAN/control
	@chmod 755 -R $(ROOT)/DEBIAN
	@dpkg-deb --build --root-owner-group $(ROOT)
