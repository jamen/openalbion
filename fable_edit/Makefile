PATH := node_modules/.bin:$(PATH)

.PHONY: all test dist

all:
	rollup --config
	sass src/style.scss out/style.css --no-source-map
	electron-build-env neon build --release
	cp native/index.node out/native.node
	cp src/index.html out/index.html

test: all
	electron .

dist: all
	electron-packager .