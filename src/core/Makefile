TESTS = test_queue test_tree

default: base

base.o : base.c base.h gc.h queue.h tree.h vec.h
	cc -c base.c -o base.o

base : base.o gc.o tree.o
	cc base.o gc.o tree.o -o base

gc.o : gc.c gc.h base.h vec.h tree.h queue.h
	cc -c gc.c -o gc.o

tree.o : tree.c tree.h vec.h
	cc -c tree.c -o tree.o

test_queue : test_queue.c queue.h
	cc test_queue.c -o test_queue

test_tree : test_tree.c tree.h tree.o
	cc test_tree.c tree.o -o test_tree

.PHONY: clean test

test : $(TESTS)
	@for t in $(TESTS); do \
		./$$t; \
	done

clean :
	-rm -f base.o
	-rm -f base
	-rm -f gc.o
	-rm -f tree.o
	-rm -f test_tree
	-rm -f test_queue
