# Build C++ client example
# Process with GNU make

all: test

check: all
	./test

HEADER := ../include/mp4parse.h
CXXFLAGS = -g -Wall -std=c++11 -I$(dir $(HEADER))

# Printing linker libraries by default was removed in rustc 1.23, so we need to
# request them explicitly.
RUSTC_MINOR_VERSION := $(shell rustc --version | cut -f2 -d.)
RUSTC_MINOR_LT_23 := $(shell [ $(RUSTC_MINOR_VERSION) -lt 23 ] && echo true)

ifeq ($(RUSTC_MINOR_LT_23), true)
PRINT_NATIVE_STATIC_LIBS :=
else
PRINT_NATIVE_STATIC_LIBS := --print native-static-libs
endif

CRATE_DIR := $(OUT_DIR)/../../../deps

libmp4parse.a libmp4parse.a.out : ../src/lib.rs
	rustc -g --crate-type staticlib --crate-name mp4parse \
	  --emit dep-info,link=$@ \
	  $(PRINT_NATIVE_STATIC_LIBS) \
	  -L $(CRATE_DIR) $< \
	  2> libmp4parse.a.out || cat libmp4parse.a.out >&2

-include mp4parse.d

ifeq ($(RUSTC_MINOR_LT_23), true)
test: RUST_LIBS = $(shell awk '/^note: library: / {print "-l"$$3}' libmp4parse.a.out)
else
test: RUST_LIBS = $(shell sed -n 's/^note: native-static-libs: //p' libmp4parse.a.out)
endif
test: test.cc libmp4parse.a $(HEADER)
	$(CXX) $(CXXFLAGS) -c $(filter %.cc,$^)
	$(CXX) $(CXXFLAGS) -o $@ *.o libmp4parse.a $(RUST_LIBS)

clean:
	$(RM) test
	$(RM) *.a.out
	$(RM) *.o *.a
	$(RM) *.d
