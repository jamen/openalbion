PATH := node_modules/.bin:$(PATH)
SHELL := /bin/bash

all:
	rm -rf out
	mkdir -p out
	electron-build-env neon build --release
	rollup --config
	sass app/style.scss out/style.css
	cp -r ../../target/i686-pc-windows-msvc/debug/fable_cheat.dll ../../target/i686-pc-windows-msvc/debug/fable_injector_32bit_helper.exe native/index.node app/static app/index.html out

dev: export NODE_ENV=development
dev: all
	electron .