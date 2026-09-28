lima.mk: ;

VM    = cc1
MOUNT = .mounts = [{"location": "$(CURDIR)", "mountPoint": "/work", "writable": true}]
LIMA  = limactl shell --shell /usr/bin/zsh --workdir /work $(VM)

up:
	limactl start --name=$(VM) --tty=false --set '$(MOUNT)' lima.yaml

start:
	limactl start $(VM)

stop:
	limactl stop $(VM)

down:
	limactl delete -f $(VM)

run:
	$(LIMA)

%: .FORCE
	$(LIMA) zsh -ic 'make $@'

.FORCE:
.PHONY: up start stop down run .FORCE
