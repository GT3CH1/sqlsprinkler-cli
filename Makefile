UNAME := $(shell uname -m)
VERSION := 0.1-2
ROOT := sqlsprinkler_$(VERSION)_$(ARCH)
default:
	$(MAKE) build

check:
	@echo "Checking for armv7l on" $(UNAME)
    ifeq ($(UNAME), x86_64)
	    @cross check --target armv7-unknown-linux-gnueabihf
    else
		cargo check
    endif

build:
	@echo "Building for armv7l on" $(UNAME)
    ifeq ($(UNAME), x86_64)
		$(MAKE) build-armv7
    else
		@cargo build
    endif

build-armv7:
	@cross build --target armv7-unknown-linux-gnueabihf

build-arm:
	@cross build --target arm-unknown-linux-gnueabihf

install-service:
	cp -v systemd/sqlsprinkler-daemon.service /etc/systemd/system

install: build install-service
	cp -v target/armv7-unknown-linux-gnueabihf/debug/sqlsprinkler-cli /usr/bin/sqlsprinkler
	install -Dm755 conf/sqlsprinkler.conf /etc/sqlsprinkler/sqlsprinkler.conf


deb:
    ifeq ($(RELEASE),true)
	RELEASE_FLAG := --release
	dir =
    endif
    ifeq ($(filter arm armv7,$(ARCH)),)
		$(error Architecture not supported, valid architectures are arm | armv7)
    endif
    ifeq ($(ARCH),arm)
		@ $(MAKE) build-arm
    endif
    ifeq ($(ARCH),armv7)
		@ $(MAKE) build-armv7
    endif
	@install -dm755 $(ROOT)
	@install -Dm755 target/$(ARCH)-unknown-linux-gnueabihf/debug/sqlsprinkler-cli $(ROOT)/usr/bin/sqlsprinkler
	@install -Dm755 conf/sqlsprinkler.conf $(ROOT)/etc/sqlsprinkler/sqlsprinkler.conf
	@install -Dm755 systemd/sqlsprinkler-daemon.service $(ROOT)/lib/systemd/system/sqlsprinkler-daemon.service
	@install -dm755 $(ROOT)/DEBIAN
	@touch $(ROOT)/DEBIAN/control
	@install -Dm755 conf/preinst $(ROOT)/DEBIAN/preinst
	@install -Dm755 conf/postinst $(ROOT)/DEBIAN/postinst
	@install -Dm755 conf/conffiles $(ROOT)/DEBIAN/conffiles
    ifeq ($(ARCH),armv7)
		@echo "Package: sqlsprinkler\n\
Version: $(VERSION)\n\
Architecture: armhf\n\
Maintainer: Gavin Pease <gavinpease@gmail.com>\n\
Description: The command line and daemon for sqlsprinkler" > $(ROOT)/DEBIAN/control
    endif
    ifeq ($(ARCH),arm)
		@echo "Package: sqlsprinkler\n\
Version: $(VERSION)\n\
Architecture: $(ARCH)\n\
Maintainer: Gavin Pease <gavinpease@gmail.com>\n\
Description: The command line and daemon for sqlsprinkler" > $(ROOT)/DEBIAN/control
    endif
	@chmod 755 -R $(ROOT)/DEBIAN
	@dpkg-deb --build --root-owner-group $(ROOT)
