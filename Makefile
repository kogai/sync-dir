NAME := sync-dir
BIN := ./target/release/$(NAME)
SRC := $(shell find ./src -type f -name '*.rs')
OS := $(shell uname)
VERSION := $(shell cat Cargo.toml | grep version | head -n1 | sed -e 's/version\ =\ \"\(.*\)\"/\1/')

bin/$(OS)/$(NAME): Cargo.toml $(SRC)
	# docker build -t $(NAME) .
	# docker run --rm -v `pwd`/target:/app/target -t $(NAME)
	cargo build --release
	mkdir -p bin/$(OS)
	cp target/release/$(NAME) bin/$(OS)/$(NAME)

.PHONY: test
test: init
	cargo build --release --features debug
	find fixture > fixture.text
	$(BIN) -s ./fixture/a ./fixture/b
	cat fixture/b/1.file # expect => do not overwrite
	echo "-----" >> fixture.text
	find fixture >> fixture.text

	sleep 0.5
	rm fixture/a/4.file
	
	$(BIN) -s ./fixture/a ./fixture/b
	echo "-----" >> fixture.text
	find fixture >> fixture.text
	cat fixture/a/1.file # expect => do not overwrite
	git status -s fixture.text

.PHONY: init
init: clean
	mkdir fixture
	mkdir fixture/a
	mkdir fixture/b
	mkdir fixture/a/aa

	touch fixture/b/2.file
	touch fixture/a/aa/3.file

	touch fixture/a/4.file
	touch fixture/b/4.file

	echo "1" > fixture/a/1.file
	echo "do not overwrite" > fixture/b/1.file
	echo "2" > fixture/b/2.file
	echo "3" > fixture/a/aa/3.file

	touch -d "9 days ago" fixture/a/1.file
	touch -d "1 days ago" fixture/b/1.file

.PHONY: release
release:
	git tag -af "v${VERSION}" -m ""
	git push
	git push --follow-tags

.PHONY: clean
clean:
	rm -f ./.sync-dir.conf
	rm -rf fixture
