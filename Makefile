.PHONY: run
run: init
	cargo run

.PHONY: init
init: clean
	mkdir fixture
	mkdir fixture/a
	mkdir fixture/b
	mkdir fixture/a/aa

	touch fixture/a/1.file
	touch fixture/b/2.file
	touch fixture/a/aa/3.file

	echo "1" > fixture/a/1.file
	echo "2" > fixture/b/2.file
	echo "3" > fixture/a/aa/3.file

.PHONY: clean
clean:
	rm -r fixture
