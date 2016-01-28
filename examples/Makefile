# Build C++ client example
# Process with GNU make

all: test

check: all
	./test

CXXFLAGS = -g -Wall -std=c++1y -I../include
RUST_LIBS = -ldl -lpthread -lrt -lgcc_s -lpthread -lc -lm

RUST_SRCS := ../src/lib.rs

libmp4parse.a libmp4parse.a.out : $(RUST_SRCS)
	rustc -g --crate-type staticlib -o libmp4parse.a -L ../target/debug/deps/ $^ 2> libmp4parse.a.out

test: RUST_LIBS = $(shell awk '/^note: library: / {print "-l"$$3}' libmp4parse.a.out)
test: dump.cc libmp4parse.a
	$(CXX) $(CXXFLAGS) -c $(filter %.cc,$^)
	$(CXX) $(CXXFLAGS) -o $@ *.o libmp4parse.a $(RUST_LIBS)

clean:
	$(RM) test
	$(RM) *.a.out
	$(RM) *.o *.a
