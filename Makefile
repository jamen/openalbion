PATH := $(PATH):node_modules/.bin
SHELL := /bin/bash

.SILENT:
.ONESHELL:
.PHONY: all clean js css html assets start

all: clean js css html assets

clean:
	rm -rf dist

js:
	rollup --config

css:
	cp src/global.css dist/global.css

html:
	cp src/index.html dist

assets:
	cp -r src/assets dist/assets

start: all
	electron ./ $(ARGS)