CC=gcc
CFLAGS=-Wall -Wextra
release:CFLAGS+=-DNDEBUG -O3
release:debouncer
debug:CFLAGS+=-g -DDEBUG -O0
debug:debouncer
debouncer:debouncer.c
	$(CC) $(CFLAGS) $^ -o $@
