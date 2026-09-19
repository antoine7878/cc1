docker.mk: ;

CONTAINER = linux-amd64-cont
IMAGE     = linux-amd64-env

CROSS_CONTAINER = linux-cross-cont
CROSS_IMAGE     = linux-cross-env

build:
	docker build --platform linux/amd64 -t $(IMAGE) .

up: build
	docker run --platform linux/amd64 -v ".:/work" -w /work --name $(CONTAINER) -d $(IMAGE)

down:
	docker rm -f $(CONTAINER)

run:
	docker exec -it $(CONTAINER) zsh

rre: down build up run

# ----- i386 cross toolchain, used by `make ctest` on non-x86_64 hosts -----

cross-build:
	docker build -f Dockerfile.cross -t $(CROSS_IMAGE) .

cross-up: cross-build
	docker run --name $(CROSS_CONTAINER) -d $(CROSS_IMAGE)

cross-down:
	docker rm -f $(CROSS_CONTAINER)

%: .FORCE
	docker exec $(CONTAINER) make $@

.FORCE:
.PHONY: build up down run rre cross-build cross-up cross-down .FORCE
