docker.mk: ;

CONTAINER = linux-amd64-cont
IMAGE     = linux-amd64-env

build:
	docker build --platform linux/amd64 -t $(IMAGE) .

up: build
	docker run --platform linux/amd64 -v ".:/work" -w /work --name $(CONTAINER) -d $(IMAGE)

down:
	docker rm -f $(CONTAINER)

run:
	docker exec -it $(CONTAINER) zsh

rre: down build up run

%: .FORCE
	docker exec $(CONTAINER) make $@

.FORCE:
.PHONY: build up down run .FORCE
