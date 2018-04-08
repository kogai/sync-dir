.PHONY: run
run: init
	ls -R fixture > fixture.text
	cargo run
	cat fixture/b/1.file
	echo "-----" >> fixture.text
	ls -R fixture >> fixture.text

.PHONY: init
init: clean
	mkdir fixture
	mkdir fixture/a
	mkdir fixture/b
	mkdir fixture/a/aa

	touch -d "9 days ago" fixture/a/1.file
	touch -d "1 days ago" fixture/b/1.file
	touch fixture/b/2.file
	touch fixture/a/aa/3.file

	touch fixture/a/4.file
	touch fixture/b/4.file

	sleep 0.1
	rm fixture/a/4.file

	echo "1" > fixture/a/1.file
	echo "do not overwrite" > fixture/b/1.file
	echo "2" > fixture/b/2.file
	echo "3" > fixture/a/aa/3.file

.PHONY: clean
clean:
	rm -r fixture
