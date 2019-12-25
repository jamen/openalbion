PATH := node_modules/.bin:$(PATH)
SHELL := /bin/bash

all:
	mkdir -p out
	electron-build-env neon build --release
	rollup --config
	sass app/style.scss out/style.css
	cp -r ../../target/release/fable_cheat.dll native/index.node app/static app/index.html out

dev: export NODE_ENV=development
dev: all
	electron .