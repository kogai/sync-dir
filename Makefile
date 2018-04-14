NAME := sync-dir
SRC := $(shell find ./src -type f -name '*.rs')

bin/$(NAME): Cargo.toml $(SRC)
	docker build -t $(NAME) .
	docker run --rm -v `pwd`/target:/app/target -t $(NAME)
	cp target/release/$(NAME) bin/$(NAME)

.PHONY: run
run: init
	find fixture > fixture.text
	cargo run -- -s ./fixture/a ./fixture/b
	cat fixture/b/1.file # expect => do not overwrite
	echo "-----" >> fixture.text
	find fixture >> fixture.text

	sleep 0.1
	rm fixture/a/4.file
	
	cargo run -- -s ./fixture/a ./fixture/b
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

.PHONY: clean
clean:
	rm ~/.sync-dir.conf
	rm -r fixture
