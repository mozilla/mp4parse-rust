/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include <cassert>
#include <cstdint>
#include <cstdio>
#include <vector>

extern "C" bool read_box_from_buffer(uint8_t *buffer, size_t size);


void test_arg_validation()
{
  bool rv;
  rv = read_box_from_buffer(nullptr, 0);
  assert(!rv);

  size_t len = 4097;
  rv = read_box_from_buffer(nullptr, len);
  assert(!rv);

  std::vector<uint8_t> buf;
  rv = read_box_from_buffer(buf.data(), buf.size());
  assert(!rv);

  buf.reserve(len);
  rv = read_box_from_buffer(buf.data(), buf.size());
  assert(!rv);
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

  bool rv = read_box_from_buffer(buf.data(), buf.size());
  assert(!rv); // Expected fail: need to trap eof.
}

int main(int argc, char* argv[])
{
  test_arg_validation();

  for (auto i = 1; i < argc; ++i) {
    read_file(argv[i]);
  }

  return 0;
}
