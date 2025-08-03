CC=gcc
CFLAGS=-Wall -Wextra
all:debouncer
debug:CFLAGS+=-DDEBUG -g
debug:debouncer
debouncer:debouncer.c
	$(CC) $(CFLAGS) $^ -o $@
