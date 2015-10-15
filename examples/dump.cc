/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include <cassert>
#include <cstdint>
#include <cstdio>
#include <vector>

extern "C" int32_t read_box_from_buffer(uint8_t *buffer, size_t size);


void test_arg_validation()
{
  int32_t rv;
  rv = read_box_from_buffer(nullptr, 0);
  assert(rv < 0);

  size_t len = 4097;
  rv = read_box_from_buffer(nullptr, len);
  assert(rv < 0);

  std::vector<uint8_t> buf;
  rv = read_box_from_buffer(buf.data(), buf.size());
  assert(rv < 0);

  buf.reserve(len);
  rv = read_box_from_buffer(buf.data(), buf.size());
  assert(rv < 0);
}

void read_file(const char* filename)
{
  FILE* f = fopen(filename, "rb");
  assert(f != nullptr);

  size_t len = 4096;
  std::vector<uint8_t> buf(len);
  size_t read = fread(buf.data(), sizeof(decltype(buf)::value_type), buf.size(), f);
  buf.resize(read);
  fclose(f);

  fprintf(stderr, "Parsing %lu byte buffer.\n", (unsigned long)read);
  int32_t rv = read_box_from_buffer(buf.data(), buf.size());
  assert(rv >= 0);
  fprintf(stderr, "%d tracks returned to C code.\n", rv);
}

int main(int argc, char* argv[])
{
  test_arg_validation();

  for (auto i = 1; i < argc; ++i) {
    read_file(argv[i]);
  }

  return 0;
}
