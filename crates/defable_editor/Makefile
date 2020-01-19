PATH := node_modules/.bin:$(PATH)
SHELL := /bin/bash

.PHONY: all clean addon js css assets html post

all: clean addon js css assets html post

clean:
	rm -rf out
	mkdir out

addon:
	electron-build-env neon build --release
	cp native/index.node out/defable_editor.node

js:
	rollup --config

css:
	sass app/style.scss out/style.css --no-source-map --style=compressed

assets:
	cp app/package.json out/package.json
	cp -r app/assets out/assets

html:
	mustache app/template.js app/index.html.tpl > out/index.html
ifneq ($(NODE_ENV),"development")
	html-minifier out/index.html --collapse-whitespace --minify-css > out/index.min.html
	mv out/index.min.html out/index.html
endif

post:
	rm -rf out/style.css out/app.js out/app.ssr.js out/app.css

start: export NODE_ENV=development
start: all
	electron ./out

start-prod: export NODE_ENV=production
start-prod: all
	electron ./out

package: export NODE_ENV=production
package: all
	rm -rf build
	mkdir build
	electron-packager ./out defable_editor \
		--asar \
		--platform=win32 \
		--arch=all \
		--electron-version=4.2.12 \
		--out build