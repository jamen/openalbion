PATH := node_modules/.bin:$(PATH)
SHELL := /bin/bash

all:
	mkdir -p out
	cp ../../target/release/fable_cheat.dll out
	electron-build-env neon build --release
	cp native/index.node app/index.html out
	rollup --config
	sass app/style.scss out/style.css

dev: export NODE_ENV=development
dev: all
	electron .