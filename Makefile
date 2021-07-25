UNAME := $(shell uname -m)
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
		@cross build --target armv7-unknown-linux-gnueabihf
    else
		@cargo build
    endif

build-pi0:
	@cross build --target arm-unknown-linux-gnueabihf

install-service:
	cp -v systemd/sqlsprinkler-daemon.service /etc/systemd/system

install: build install-service
	cp -v target/armv7-unknown-linux-gnueabihf/debug/sqlsprinkler-cli /usr/bin/sqlsprinkler
	install -Dm755 conf/sqlsprinkler.conf /etc/sqlsprinkler/sqlsprinkler.conf
