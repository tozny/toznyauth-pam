ROOT_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

all: dist/toznyauth_pam.so

dist:
	mkdir dist

dist/toznyauth_pam.so: Cargo.toml Cargo.lock $(shell find src -name '*.rs')
	docker build -t toznyauth_pam .
	docker run -i -v $(ROOT_DIR)/dist:/dist toznyauth_pam sh -c \
		'cp /code/target/libtoznyauth_pam*.so /dist/toznyauth_pam.so'
