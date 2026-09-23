docker.mk: ;

CONTAINER = linux-amd64-cont
IMAGE     = linux-amd64-env

CROSS_CONTAINER = linux-cross-cont
CROSS_IMAGE     = linux-cross-env

build:
	docker build --platform linux/amd64 -t $(IMAGE) .
	docker build -f Dockerfile.cross -t $(CROSS_IMAGE) .

up:
	docker run --platform linux/amd64 -v ".:/work" -w /work --name $(CONTAINER) -d $(IMAGE)
	docker run --name $(CROSS_CONTAINER) -d $(CROSS_IMAGE)

down:
	docker rm -f $(CONTAINER)
	docker rm -f $(CROSS_CONTAINER)

run:
	docker exec -it $(CONTAINER) zsh

rre: down build up run

%: .FORCE
	docker exec $(CONTAINER) make $@

.FORCE:
.PHONY: build up down run rre cross-build cross-up cross-down .FORCE
